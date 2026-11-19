; Block indentation (Sub/If/For ...) is handled by increase_indent_pattern and
; decrease_indent_pattern in config.toml, because the grammar hides keywords.
(_ "(" ")" @end) @indent
(_ "{" "}" @end) @indent
