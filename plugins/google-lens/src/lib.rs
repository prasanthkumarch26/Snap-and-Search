use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::io::Write;

/// Uploads an image to Google Lens by writing a local HTML file that the browser
/// submits natively as a multipart form.
///
/// `image_bytes` should be JPEG for speed (smaller → faster atob() and upload).
/// `mime_type` should be `"image/jpeg"` or `"image/png"`.
pub fn upload_and_open(image_bytes: Vec<u8>, mime_type: &str) -> Result<(), String> {
    println!("Preparing {} bytes for Google Lens...", image_bytes.len());

    // 1. Encode the PNG bytes to base64
    let base64_image = STANDARD.encode(&image_bytes);

    // 2. Generate the HTML page.
    //    - We use atob() to decode base64 → binary string → Uint8Array → Blob → File.
    //    - This is 100% in-memory JavaScript, no fetch() or file:// access needed.
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Searching with Google Lens...</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Google Sans', system-ui, sans-serif;
            display: flex; flex-direction: column;
            justify-content: center; align-items: center;
            height: 100vh;
            background: #202124;
            color: #e8eaed;
            gap: 20px;
        }}
        .spinner {{
            width: 44px; height: 44px;
            border: 4px solid #3c4043;
            border-top-color: #8ab4f8;
            border-radius: 50%;
            animation: spin 0.9s linear infinite;
        }}
        @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
        p {{ font-size: 15px; opacity: 0.8; }}
    </style>
</head>
<body>
    <div class="spinner"></div>
    <p>Searching with Google Lens...</p>

    <form id="lens-form"
          action="https://lens.google.com/v3/upload?ep=ccm&s=4&st=0"
          method="POST"
          enctype="multipart/form-data"
          style="display:none;">
        <input type="file" name="encoded_image" id="file-input">
    </form>

    <script>
        // Decode the base64 PNG completely in-memory using atob().
        // No fetch(), no file:// access — works regardless of browser security settings.
        (function() {{
            try {{
                var b64 = "{b64}";
                
                // atob gives us a binary string; convert to Uint8Array
                var binaryStr = atob(b64);
                var bytes = new Uint8Array(binaryStr.length);
                for (var i = 0; i < binaryStr.length; i++) {{
                    bytes[i] = binaryStr.charCodeAt(i);
                }}

                // Wrap in a Blob then a File
                var blob = new Blob([bytes], {{ type: "{mime}" }});
                var file = new File([blob], "capture.{ext}", {{ type: "{mime}" }});

                // Attach to the hidden form input and submit
                var dt = new DataTransfer();
                dt.items.add(file);
                document.getElementById("file-input").files = dt.files;
                document.getElementById("lens-form").submit();
            }} catch(err) {{
                document.body.innerHTML =
                    "<h3 style='color:#f28b82'>Failed to process image</h3><pre>" + err + "</pre>";
            }}
        }})();
    </script>
</body>
</html>"#,
        b64 = base64_image,
        mime = mime_type,
        ext = if mime_type == "image/jpeg" { "jpg" } else { "png" }
    );

    // 3. Write the HTML to the temp directory
    let temp_path = std::env::temp_dir().join("lens_upload.html");
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp HTML file: {}", e))?;
    file.write_all(html_content.as_bytes())
        .map_err(|e| format!("Failed to write temp HTML file: {}", e))?;

    // 4. Open the HTML file in the default browser
    let file_url = format!(
        "file:///{}",
        temp_path.display().to_string().replace('\\', "/")
    );

    println!("Opening browser for Lens upload...");
    webbrowser::open(&file_url)
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(())
}