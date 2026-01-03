use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::journal::entry::JournalEntry;
use crate::journal::filesystem;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[derive(Deserialize)]
struct DateQuery {
    date: Option<String>,
}

#[derive(Deserialize)]
struct CreateEntryRequest {
    date: Option<String>,
    content: String,
}

#[derive(Serialize)]
struct EntryResponse {
    date: String,
    content: String,
    exists: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(config: &Config) -> Result<()> {
    let state = AppState {
        config: Arc::new(config.clone()),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/entry", get(get_entry))
        .route("/api/entry", post(create_entry))
        .with_state(state);

    let addr = "0.0.0.0:3030";
    println!("🚀 Journal web server starting on http://{}", addr);
    println!("📱 Access from your phone at http://<your-local-ip>:3030");
    println!("Press Ctrl+C to stop the server");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn serve_index() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Journal Entry</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            border-radius: 20px;
            padding: 30px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }

        h1 {
            color: #333;
            margin-bottom: 10px;
            font-size: 28px;
        }

        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 14px;
        }

        .date-selector {
            margin-bottom: 20px;
        }

        label {
            display: block;
            margin-bottom: 8px;
            color: #555;
            font-weight: 500;
        }

        input[type="date"] {
            width: 100%;
            padding: 12px;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.3s;
        }

        input[type="date"]:focus {
            outline: none;
            border-color: #667eea;
        }

        textarea {
            width: 100%;
            min-height: 400px;
            padding: 15px;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 16px;
            font-family: 'Courier New', monospace;
            resize: vertical;
            transition: border-color 0.3s;
            line-height: 1.6;
        }

        textarea:focus {
            outline: none;
            border-color: #667eea;
        }

        .button-group {
            display: flex;
            gap: 10px;
            margin-top: 20px;
        }

        button {
            flex: 1;
            padding: 14px 24px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s;
        }

        .btn-primary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }

        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4);
        }

        .btn-secondary {
            background: #f0f0f0;
            color: #333;
        }

        .btn-secondary:hover {
            background: #e0e0e0;
        }

        .message {
            margin-top: 20px;
            padding: 12px;
            border-radius: 8px;
            display: none;
        }

        .message.success {
            background: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }

        .message.error {
            background: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }

        .message.show {
            display: block;
        }

        @media (max-width: 600px) {
            .container {
                padding: 20px;
            }

            h1 {
                font-size: 24px;
            }

            .button-group {
                flex-direction: column;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📔 Daily Journal</h1>
        <p class="subtitle">Write your thoughts and reflections</p>

        <div class="date-selector">
            <label for="date">Entry Date:</label>
            <input type="date" id="date" value="">
        </div>

        <label for="content">Entry Content:</label>
        <textarea id="content" placeholder="Write your journal entry here..."></textarea>

        <div class="button-group">
            <button class="btn-secondary" onclick="loadEntry()">Load Entry</button>
            <button class="btn-primary" onclick="saveEntry()">Save Entry</button>
        </div>

        <div id="message" class="message"></div>
    </div>

    <script>
        // Set today's date by default
        const dateInput = document.getElementById('date');
        const today = new Date().toISOString().split('T')[0];
        dateInput.value = today;

        // Load entry on page load
        window.onload = () => loadEntry();

        // Load entry when date changes
        dateInput.addEventListener('change', loadEntry);

        function showMessage(text, type) {
            const message = document.getElementById('message');
            message.textContent = text;
            message.className = `message ${type} show`;
            setTimeout(() => {
                message.className = 'message';
            }, 5000);
        }

        async function loadEntry() {
            const date = dateInput.value;

            try {
                const response = await fetch(`/api/entry?date=${date}`);
                const data = await response.json();

                if (response.ok) {
                    document.getElementById('content').value = data.content;
                    if (data.exists) {
                        showMessage('Entry loaded successfully', 'success');
                    } else {
                        showMessage('New entry template loaded', 'success');
                    }
                } else {
                    showMessage(`Error: ${data.error}`, 'error');
                }
            } catch (error) {
                showMessage(`Failed to load entry: ${error.message}`, 'error');
            }
        }

        async function saveEntry() {
            const date = dateInput.value;
            const content = document.getElementById('content').value;

            if (!content.trim()) {
                showMessage('Please write something before saving', 'error');
                return;
            }

            try {
                const response = await fetch('/api/entry', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ date, content }),
                });

                const data = await response.json();

                if (response.ok) {
                    showMessage('Entry saved successfully! 🎉', 'success');
                } else {
                    showMessage(`Error: ${data.error}`, 'error');
                }
            } catch (error) {
                showMessage(`Failed to save entry: ${error.message}`, 'error');
            }
        }

        // Keyboard shortcut: Ctrl+S or Cmd+S to save
        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                saveEntry();
            }
        });
    </script>
</body>
</html>"#
            .to_string(),
    )
}

async fn get_entry(
    State(state): State<AppState>,
    Query(params): Query<DateQuery>,
) -> impl IntoResponse {
    let date = match params.date {
        Some(date_str) => match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid date format".to_string(),
                    }),
                )
                    .into_response();
            }
        },
        None => Local::now().date_naive(),
    };

    let entry_path = filesystem::get_entry_path(date, &state.config.journal_dir);
    let exists = entry_path.exists();

    let content = if exists {
        match fs::read_to_string(&entry_path) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to read entry: {}", e),
                    }),
                )
                    .into_response();
            }
        }
    } else {
        // Create entry using existing logic to get the template
        match JournalEntry::create(date, &state.config) {
            Ok(entry) => match fs::read_to_string(&entry.file_path) {
                Ok(c) => c,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to read new entry: {}", e),
                        }),
                    )
                        .into_response();
                }
            },
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to create entry: {}", e),
                    }),
                )
                    .into_response();
            }
        }
    };

    (
        StatusCode::OK,
        Json(EntryResponse {
            date: date.format("%Y-%m-%d").to_string(),
            content,
            exists,
        }),
    )
        .into_response()
}

async fn create_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateEntryRequest>,
) -> impl IntoResponse {
    let date = match payload.date {
        Some(date_str) => match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid date format".to_string(),
                    }),
                )
                    .into_response();
            }
        },
        None => Local::now().date_naive(),
    };

    // Ensure the entry structure exists
    let year = date.format("%Y").to_string().parse::<u32>().unwrap();
    let month = date.format("%m").to_string().parse::<u32>().unwrap();

    if let Err(e) = filesystem::ensure_year_dir(year, &state.config.journal_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create year directory: {}", e),
            }),
        )
            .into_response();
    }

    if let Err(e) = filesystem::ensure_month_dir(year, month, &state.config.journal_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create month directory: {}", e),
            }),
        )
            .into_response();
    }

    if let Err(e) = filesystem::create_month_readme(year, month, &state.config.journal_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create month README: {}", e),
            }),
        )
            .into_response();
    }

    let entry_path = filesystem::get_entry_path(date, &state.config.journal_dir);

    // Write the content
    if let Err(e) = fs::write(&entry_path, &payload.content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to write entry: {}", e),
            }),
        )
            .into_response();
    }

    // Update SUMMARY.md
    let summary_path = state.config.journal_dir.join("SUMMARY.md");
    if let Ok(mut summary) = crate::journal::summary::Summary::parse(&summary_path) {
        summary.add_day_entry(date);
        let _ = summary.write();
    }

    (
        StatusCode::OK,
        Json(EntryResponse {
            date: date.format("%Y-%m-%d").to_string(),
            content: payload.content,
            exists: true,
        }),
    )
        .into_response()
}
