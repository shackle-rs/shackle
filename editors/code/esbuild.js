const esbuild = require("esbuild")

const production = process.argv.includes("--production")
const watch = process.argv.includes("--watch")

const watchPlugin = {
	name: "watch-status",
	setup(build) {
		build.onStart(() => {
			console.log("[watch] Bundling extension with esbuild")
		})
		build.onEnd((result) => {
			if (result.errors.length === 0) {
				console.log("[watch] Bundle finished")
			}
		})
	},
}

const buildOptions = {
	entryPoints: ["src/main.ts"],
	bundle: true,
	external: ["vscode"],
	format: "cjs",
	platform: "node",
	target: "node16",
	outfile: "out/main.js",
	minify: production,
	sourcemap: true,
	sourcesContent: false,
	logLevel: "info",
	plugins: watch ? [watchPlugin] : [],
}

async function main() {
	if (watch) {
		const context = await esbuild.context(buildOptions)
		await context.watch()
		return
	}

	await esbuild.build(buildOptions)
}

main().catch((error) => {
	console.error(error)
	process.exit(1)
})
