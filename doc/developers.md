# Notes for Sea Developers

## File/Folder Structure

```
doc/          - Documentation, guides, etc
res/          - Resources (images and such)
samples/      - Sea samples
src/          - Compiler source code
  backend/    - Backend trait
    backends/ - Backends
  compile/    - Compiler (not the backend!)
  parse/      - Lexer and parser
  sandbox/    - Sea sandbox code
std/          - Sea standard library
tests/        - Tests for the lexer/parser/backend/compiler/whatever
```

## IDE/Editor

If you're using VSCode, I highly recommend adding these two extensions:

- https://marketplace.visualstudio.com/items?itemName=aaron-bond.better-comments
- https://marketplace.visualstudio.com/items?itemName=MohammadBaqer.better-folding

Then add this to your settings.json (user or workspace, doesn't matter):

```json
{
	...
	"better-comments.tags": {
		...
		{
			"tag": "#region",
			"color": "#3498DB",
			"strikethrough": false,
			"underline": false,
			"backgroundColor": "transparent",
			"bold": false,
			"italic": false
		},
		{
			"tag": "#endregion",
			"color": "#3498DB",
			"strikethrough": false,
			"underline": false,
			"backgroundColor": "transparent",
			"bold": false,
			"italic": false
		}
	},
	...
	"explicitFolding.rules": {
		"*": {
			"begin": "#region",
			"end": "#endregion"
		}
	}
}
```

This will give you highlighted comment regions, which are particularly nice to
have in the larger files (`parser.rs`, for example).
