# Notes for Sea Developers

## IDE/Editor

If you're using VSCode, I highly recommend adding these two extensions:

- https://marketplace.visualstudio.com/items?itemName=aaron-bond.better-comments
- https://marketplace.visualstudio.com/items?itemName=MohammadBaqer.better-folding

Then add this to your settings.json:

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
