// Add boilerplate to HTML stubs.
pub fn complete_html(stub: &str) -> String {
    format!("
        <!DOCTYPE html>
        <html>
        <head>
        <script>
        if(window.external===undefined){{
            window.external={{invoke:function(x){{window.webkit.messageHandlers.external.postMessage(x);}}}};
        }}
        function sendMessage(command,data){{
            if(data===undefined){{data=''}}
            const message = command + \" \" + data;
            external.invoke(message);
        }}
        </script>
        <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\";
          }}
        input{{
            display:block;
            width:80%;
        }}
        ul{{
            padding-left:0px;
            padding-right:10px;
        }}
        li{{
            list-style-type:none;
            margin-bottom:10px;
            border-bottom:1px solid lightgrey;
        }}
        summary{{
            cursor:pointer;
        }}
        </style>
        </head>
        <body>
        {}
        </body>
        </html>
    ",stub)
}
