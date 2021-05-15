pub fn complete_html(stub: &str) -> String {
    format!("
        <html>
        <head>
        <script>
        if(window.external===undefined){{
            window.external={{invoke:function(x){{window.webkit.messageHandlers.external.postMessage(x);}}}};
        }}
        function sendMessage(command,data){{
            const message = command + \" \" + JSON.stringify(data);
            external.invoke(message);
        }}
        </script>
        <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\";
          }}
        </style>
        </head>
        <body>
        {}
        </body>
        </html>
    ",stub)
}
