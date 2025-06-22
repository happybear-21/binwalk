use axum::{extract::{Multipart, Path, State}, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Shared state for storing extracted files (in-memory for demo; use disk for real app)
type ExtractedFiles = Arc<Mutex<HashMap<String, Vec<u8>>>>;

#[derive(Debug, Deserialize)]
struct AnalyzeOptions {
    extract: bool,
    carve: bool,
    entropy: bool,
    matryoshka: bool,
    include: Option<String>, // comma-separated
    exclude: Option<String>, // comma-separated
    threads: Option<usize>,
    directory: Option<String>,
    verbose: bool,
    quiet: bool,
}

#[derive(Debug, Serialize)]
struct AnalyzeResult {
    file_map: Vec<binwalk::signatures::common::SignatureResult>,
    extractions: HashMap<String, String>, // id -> download URL
    entropy: Option<Vec<f64>>, // stub for now
}

pub fn start_server() {
    tokio_main();
}

#[tokio::main]
async fn tokio_main() {
    let state: ExtractedFiles = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/", get(index))
        .route("/api/analyze", post(analyze))
        .route("/api/list", get(list_signatures))
        .route("/api/entropy", post(entropy))
        .route("/api/download/:id", get(download_file))
        .with_state(Arc::clone(&state));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Serving Binwalk Web UI at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Serve the HTML/JS frontend
async fn index() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang='en'>
    <head>
        <meta charset='UTF-8'>
        <meta name='viewport' content='width=device-width, initial-scale=1.0'>
        <title>Binwalk Web UI</title>
        <script src='https://cdn.plot.ly/plotly-latest.min.js'></script>
        <style>
            html, body {
                height: 100%;
            }
            body {
                min-height: 100vh;
                background: #111;
                color: #fff;
                font-family: 'Inter', 'Segoe UI', Arial, sans-serif;
                margin: 0;
                padding: 0;
                display: flex;
                flex-direction: column;
                align-items: center;
            }
            h1 {
                font-weight: 700;
                font-size: 2.1em;
                letter-spacing: -1px;
                margin: 2.2em 0 1.1em 0;
                color: #fff;
                text-align: center;
            }
            .split-container {
                display: flex;
                flex-direction: row;
                gap: 2.5em;
                width: 100vw;
                max-width: 1200px;
                justify-content: center;
                align-items: flex-start;
            }
            .split-left, .split-right {
                flex: 1 1 0;
                min-width: 320px;
                max-width: 480px;
            }
            .split-left {
                background: #18181b;
                border-radius: 12px;
                border: 1.5px solid #232329;
                box-shadow: 0 2px 12px 0 rgba(0,0,0,0.10);
                padding: 2.5em 2em 2em 2em;
                margin-bottom: 2em;
            }
            .split-right {
                background: #18181b;
                border-radius: 12px;
                border: 1.5px solid #232329;
                box-shadow: 0 2px 12px 0 rgba(0,0,0,0.10);
                padding: 2.5em 2em 2em 2em;
                margin-bottom: 2em;
                min-height: 300px;
            }
            @media (max-width: 900px) {
                .split-container {
                    flex-direction: column;
                    gap: 0;
                    align-items: stretch;
                }
                .split-left, .split-right {
                    max-width: 100vw;
                    margin-bottom: 0.5em;
                    padding: 1.2em 0.7em 1em 0.7em;
                }
                h1 {
                    font-size: 1.3em;
                    margin: 1.2em 0 0.7em 0;
                }
            }
            form {
                display: flex;
                flex-direction: column;
                gap: 1.1em;
            }
            .form-group {
                display: flex;
                flex-wrap: wrap;
                gap: 0.9em 1.5em;
                margin-bottom: 0.2em;
            }
            label {
                font-size: 1em;
                color: #e5e5e5;
                display: flex;
                align-items: center;
                gap: 0.3em;
                cursor: pointer;
            }
            input[type='file'] {
                margin-bottom: 0.5em;
                color-scheme: dark;
            }
            input[type='text'], input[type='number'] {
                padding: 0.6em 0.9em;
                border: 1.5px solid #232329;
                border-radius: 8px;
                font-size: 1em;
                background: #111;
                color: #fff;
                transition: border 0.2s;
            }
            input[type='text']:focus, input[type='number']:focus {
                border: 1.5px solid #fff;
                outline: none;
            }
            input[type='checkbox'] {
                accent-color: #fff;
            }
            button, .btn {
                background: #fff;
                color: #111;
                border: 1.5px solid #232329;
                padding: 0.7em 1.4em;
                font-size: 1.1em;
                border-radius: 8px;
                font-weight: 600;
                cursor: pointer;
                transition: background 0.2s, color 0.2s, border 0.2s;
                margin-top: 0.5em;
            }
            button:hover, .btn:hover {
                background: #232329;
                color: #fff;
                border: 1.5px solid #fff;
            }
            #list-sigs {
                background: #18181b;
                color: #fff;
                border: 1.5px solid #232329;
                margin-top: 1.2em;
                margin-bottom: 0.7em;
                box-shadow: none;
                font-size: 1em;
            }
            #list-sigs:hover {
                background: #232329;
                color: #fff;
                border: 1.5px solid #fff;
            }
            .result, #signatures, #entropy-plot {
                background: #18181b;
                border-radius: 10px;
                border: 1.5px solid #232329;
                box-shadow: 0 1px 6px 0 rgba(0,0,0,0.08);
                padding: 1.3em 1em 1em 1em;
                margin-top: 1.1em;
                max-width: 600px;
                width: 100%;
                word-break: break-word;
                color: #fff;
            }
            table {
                width: 100%;
                border-collapse: collapse;
                margin-top: 0.7em;
                font-size: 0.97em;
                background: #18181b;
                color: #fff;
            }
            th, td {
                padding: 0.5em 0.5em;
                border-bottom: 1px solid #232329;
                text-align: left;
            }
            th {
                background: #232329;
                font-weight: 600;
                color: #fff;
            }
            tr:last-child td {
                border-bottom: none;
            }
            ul {
                margin: 0.5em 0 0 1.2em;
                padding: 0;
            }
            ul li {
                margin-bottom: 0.3em;
            }
            .hidden {
                display: none;
            }
            @media (max-width: 600px) {
                .container, .result, #signatures, #entropy-plot {
                    max-width: 98vw;
                    padding: 1.2em 0.4em 1em 0.4em;
                }
                h1 {
                    font-size: 1.3em;
                }
            }
        </style>
    </head>
    <body>
        <h1>Binwalk Web UI</h1>
        <div class="split-container">
            <div class="split-left">
                <form id='analyze-form'>
                    <input type='file' name='file' required />
                    <div class='form-group'>
                        <label><input type='checkbox' name='extract' /> Extract</label>
                        <label><input type='checkbox' name='carve' /> Carve</label>
                        <label><input type='checkbox' name='entropy' /> Entropy</label>
                        <label><input type='checkbox' name='matryoshka' /> Recursive</label>
                    </div>
                    <div class='form-group'>
                        <label><input type='checkbox' name='verbose' /> Verbose</label>
                        <label><input type='checkbox' name='quiet' /> Quiet</label>
                    </div>
                    <input type='text' name='include' placeholder='Include signatures (comma separated)' />
                    <input type='text' name='exclude' placeholder='Exclude signatures (comma separated)' />
                    <input type='number' name='threads' placeholder='Threads' min='1' />
                    <input type='text' name='directory' placeholder='Output directory' />
                    <button type='submit'>Analyze</button>
                </form>
                <button id='list-sigs'>List Signatures</button>
            </div>
            <div class="split-right">
                <div id='signatures' class='result'></div>
                <div id='result' class='result'></div>
                <div id='entropy-plot' class='result hidden'></div>
            </div>
        </div>
        <script>
        document.getElementById('analyze-form').onsubmit = async function(e) {
            e.preventDefault();
            const form = e.target;
            const data = new FormData(form);
            const opts = {};
            for (const el of form.elements) {
                if (el.name && el.type !== 'file' && el.type !== 'submit') {
                    if (el.type === 'checkbox') opts[el.name] = el.checked;
                    else opts[el.name] = el.value;
                }
            }
            data.append('options', JSON.stringify(opts));
            const res = await fetch('/api/analyze', { method: 'POST', body: data });
            const json = await res.json();
            let html = '<h2>Results</h2>';
            if (json.file_map && json.file_map.length) {
                html += '<table border=1><tr><th>Offset</th><th>Description</th><th>Size</th></tr>';
                for (const r of json.file_map) {
                    html += `<tr><td>${r.offset}</td><td>${r.description}</td><td>${r.size}</td></tr>`;
                }
                html += '</table>';
            }
            if (json.extractions && Object.keys(json.extractions).length) {
                html += '<h3>Extractions</h3><ul>';
                for (const [id, url] of Object.entries(json.extractions)) {
                    html += `<li><a href="${url}">Download ${id}</a></li>`;
                }
                html += '</ul>';
            }
            document.getElementById('result').innerHTML = html;
            if (json.entropy) {
                document.getElementById('entropy-plot').classList.remove('hidden');
                Plotly.newPlot('entropy-plot', [{ y: json.entropy, type: 'scatter' }], { title: 'Entropy' });
            } else {
                document.getElementById('entropy-plot').classList.add('hidden');
            }
        };
        document.getElementById('list-sigs').onclick = async function() {
            const res = await fetch('/api/list');
            const json = await res.json();
            let html = '<h2>Supported Signatures</h2>';
            if (json.signatures && json.signatures.length) {
                html += '<ul>';
                for (const sig of json.signatures) {
                    html += `<li><b>${sig.name}</b>: ${sig.description}</li>`;
                }
                html += '</ul>';
            } else {
                html += '<p>No signatures found.</p>';
            }
            document.getElementById('signatures').innerHTML = html;
        };
        </script>
    </body>
    </html>
    "#)
}

// Handle file upload and analysis
async fn analyze(
    State(state): State<ExtractedFiles>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    use binwalk::Binwalk;
    let mut file_bytes = Vec::new();
    let mut opts: Option<AnalyzeOptions> = None;
    let mut filename = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("");
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            file_bytes = field.bytes().await.unwrap().to_vec();
        } else if name == "options" {
            let v = field.bytes().await.unwrap();
            opts = serde_json::from_slice(&v).ok();
        }
    }
    let opts = opts.unwrap_or(AnalyzeOptions {
        extract: false, carve: false, entropy: false, matryoshka: false,
        include: None, exclude: None, threads: None, directory: None,
        verbose: false, quiet: false,
    });
    // Run binwalk analysis
    // NOTE: verbose and quiet options are parsed, but Binwalk::configure does not currently use them directly.
    // If you add support in Binwalk, pass opts.verbose and opts.quiet here.
    let binwalker = Binwalk::configure(
        filename.clone(),
        opts.directory.clone(),
        opts.include.as_ref().map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        opts.exclude.as_ref().map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        None,
        false,
    ).unwrap();
    let results = binwalker.analyze_buf(&file_bytes, filename.clone().unwrap_or("upload.bin".to_string()), opts.extract);
    // Save extractions for download (use output_directory)
    let mut extractions = HashMap::new();
    for (id, extraction) in &results.extractions {
        // Try to find a file in the output_directory
        if !extraction.output_directory.is_empty() {
            let dir = &extraction.output_directory;
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(data) = fs::read(entry.path()) {
                        let uuid = Uuid::new_v4().to_string();
                        state.lock().await.insert(uuid.clone(), data);
                        extractions.insert(id.clone(), format!("/api/download/{}", uuid));
                    }
                }
            }
        }
    }
    // (Stub) Entropy
    let entropy = if opts.entropy {
        Some(vec![]) // TODO: Call entropy analysis and return data
    } else { None };
    let resp = AnalyzeResult {
        file_map: results.file_map,
        extractions,
        entropy,
    };
    Json(resp)
}

// List supported signatures/extractors
async fn list_signatures() -> impl IntoResponse {
    // Try to get real signature data from binwalk
    // This assumes binwalk::signatures::common::Signature::all() or similar is available
    #[derive(serde::Serialize)]
    struct SignatureInfo {
        name: String,
        description: String,
    }
    let mut sigs: Vec<SignatureInfo> = Vec::new();
    #[allow(unused_imports)]
    use binwalk::signatures::common::Signature;
    // If you have an API to list all signatures, use it, otherwise leave as TODO
    // Example:
    // for sig in Signature::all() {
    //     sigs.push(SignatureInfo { name: sig.name.clone(), description: sig.description.clone() });
    // }
    // For now, return empty if not implemented
    Json(serde_json::json!({ "signatures": sigs }))
}

// Entropy analysis endpoint (stub)
async fn entropy() -> impl IntoResponse {
    // TODO: Implement entropy analysis
    Json(serde_json::json!({ "entropy": [] }))
}

// Download extracted file
async fn download_file(
    Path(id): Path<String>,
    State(state): State<ExtractedFiles>,
) -> impl IntoResponse {
    use axum::http::{StatusCode, header};
    let map = state.lock().await;
    if let Some(data) = map.get(&id) {
        (
            [(header::CONTENT_TYPE, "application/octet-stream")],
            data.clone()
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            "Not found"
        )
            .into_response()
    }
} 