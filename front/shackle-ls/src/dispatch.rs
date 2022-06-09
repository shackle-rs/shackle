use lsp_server::{ExtractError, Notification, Request, Response, ResponseError};

#[derive(Debug)]
enum State<U, T, H> {
	Unhandled(U, T),
	Handled(H),
}

#[derive(Debug)]
pub struct DispatchRequest<T> {
	state: State<Request, T, Result<Response, ExtractError<Request>>>,
}

impl<T> DispatchRequest<T> {
	pub fn new(r: Request, context: T) -> Self {
		eprintln!("got {} request #{}", r.method, r.id);
		Self {
			state: State::Unhandled(r, context),
		}
	}

	pub fn on<R, F>(self, mut f: F) -> Self
	where
		R: lsp_types::request::Request,
		F: FnMut(T, R::Params) -> Result<R::Result, ResponseError>,
	{
		match self.state {
			State::Unhandled(request, context) => {
				if request.method == R::METHOD {
					let id = request.id.clone();
					let result = serde_json::from_value(request.params)
						.map_err(|error| ExtractError::JsonError {
							method: request.method.clone(),
							error,
						})
						.map(|params| f(context, params))
						.map(|result| match result {
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
						});
					Self {
						state: State::Handled(result),
					}
				} else {
					Self {
						state: State::Unhandled(request, context),
					}
				}
			}
			State::Handled(_) => self,
		}
	}

	pub fn finish(self) -> Result<Response, ExtractError<Request>> {
		match self.state {
			State::Handled(result) => result,
			State::Unhandled(request, _) => Err(ExtractError::MethodMismatch(request)),
		}
	}
}

#[derive(Debug)]
pub struct DispatchNotification<T> {
	state: State<Notification, T, Result<(), ExtractError<Notification>>>,
}

impl<T> DispatchNotification<T> {
	pub fn new(n: Notification, context: T) -> Self {
		eprintln!("got {} notification", n.method);
		Self {
			state: State::Unhandled(n, context),
		}
	}

	pub fn on<N, F>(self, mut f: F) -> Self
	where
		N: lsp_types::notification::Notification,
		F: FnMut(T, N::Params),
	{
		match self.state {
			State::Unhandled(notification, context) => {
				if notification.method == N::METHOD {
					let result = serde_json::from_value(notification.params)
						.map_err(|error| ExtractError::JsonError {
							method: notification.method.clone(),
							error,
						})
						.map(|params| f(context, params));
					Self {
						state: State::Handled(result),
					}
				} else {
					Self {
						state: State::Unhandled(notification, context),
					}
				}
			}
			State::Handled(_) => self,
		}
	}

	pub fn finish(self) -> Result<(), ExtractError<Notification>> {
		match self.state {
			State::Handled(result) => result,
			State::Unhandled(notification, _) => Err(ExtractError::MethodMismatch(notification)),
		}
	}
}
