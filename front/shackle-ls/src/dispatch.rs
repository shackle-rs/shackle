use lsp_server::{ExtractError, Message, Notification, Request, Response, ResponseError};
use shackle::hir::db::Hir;

use crate::LanguageServerDatabase;

enum RequestState<'a> {
	Unhandled {
		request: Request,
		db: &'a mut LanguageServerDatabase,
	},
	Handled(Result<(), ExtractError<Request>>),
}

pub trait RequestHandler<R: lsp_types::request::Request, T> {
	/// Run on the main thread to prepare the argument passed to the `execute` function.
	/// Usually needs to call `set_input_files` on the database.
	fn prepare(db: &mut LanguageServerDatabase, params: R::Params) -> Result<T, ResponseError>;
	/// Run in the thread pool. Can panic without crashing the language server.
	fn execute(db: &dyn Hir, data: T) -> Result<R::Result, ResponseError>;
}

pub struct DispatchRequest<'a>(RequestState<'a>);

impl<'a> DispatchRequest<'a> {
	pub fn new(request: Request, db: &'a mut LanguageServerDatabase) -> Self {
		eprintln!("got {} request #{}", request.method, request.id);
		Self(RequestState::Unhandled { request, db })
	}
	pub fn on<H, R, T>(self) -> Self
	where
		R: lsp_types::request::Request,
		H: RequestHandler<R, T>,
		T: Send + 'static,
	{
		match self.0 {
			RequestState::Unhandled { request, db } => {
				if request.method == R::METHOD {
					let id = request.id.clone();
					match serde_json::from_value(request.params) {
						Ok(params) => {
							let value = match H::prepare(db, params) {
								Ok(v) => v,
								Err(e) => {
									db.send(Message::Response(Response {
										id,
										result: None,
										error: Some(e),
									}))
									.unwrap_or_else(|e| {
										eprintln!("failed to send response: {:?}", e)
									});
									return Self(RequestState::Handled(Ok(())));
								}
							};
							db.execute_async(move |db, sender| {
								let response = match H::execute(db, value) {
									Ok(value) => Response {
										id,
										result: Some(
											serde_json::to_value(&value)
												.expect("Failed to serialize response"),
										),
										error: None,
									},
									Err(err) => Response {
										id,
										result: None,
										error: Some(err),
									},
								};
								sender
									.send(Message::Response(response))
									.unwrap_or_else(|e| {
										eprintln!("failed to send response: {:?}", e)
									});
							});
							Self(RequestState::Handled(Ok(())))
						}
						Err(error) => Self(RequestState::Handled(Err(ExtractError::JsonError {
							method: request.method.clone(),
							error,
						}))),
					}
				} else {
					Self(RequestState::Unhandled { request, db })
				}
			}
			_ => self,
		}
	}

	pub fn finish(self) -> Result<(), ExtractError<Request>> {
		match self.0 {
			RequestState::Handled(result) => result,
			RequestState::Unhandled { request, .. } => Err(ExtractError::MethodMismatch(request)),
		}
	}
}

enum NotificationState<'a> {
	Unhandled {
		notification: Notification,
		db: &'a mut LanguageServerDatabase,
	},
	Handled(Result<(), ExtractError<Notification>>),
}

pub struct DispatchNotification<'a>(NotificationState<'a>);

impl<'a> DispatchNotification<'a> {
	pub fn new(notification: Notification, db: &'a mut LanguageServerDatabase) -> Self {
		eprintln!("got {} notification", notification.method);
		Self(NotificationState::Unhandled { notification, db })
	}

	pub fn on<N, F>(self, mut f: F) -> Self
	where
		N: lsp_types::notification::Notification,
		F: FnMut(&mut LanguageServerDatabase, N::Params),
	{
		match self.0 {
			NotificationState::Unhandled { notification, db } => {
				if notification.method == N::METHOD {
					match serde_json::from_value(notification.params) {
						Ok(params) => {
							f(db, params);
							Self(NotificationState::Handled(Ok(())))
						}
						Err(error) => {
							Self(NotificationState::Handled(Err(ExtractError::JsonError {
								method: notification.method.clone(),
								error,
							})))
						}
					}
				} else {
					Self(NotificationState::Unhandled { notification, db })
				}
			}
			_ => self,
		}
	}

	pub fn finish(self) -> Result<(), ExtractError<Notification>> {
		match self.0 {
			NotificationState::Handled(result) => result,
			NotificationState::Unhandled { notification, .. } => {
				Err(ExtractError::MethodMismatch(notification))
			}
		}
	}
}
