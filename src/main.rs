use actix_files::NamedFile;
use actix_web::{
    get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};
use rebarchat::models;
use rebarchat::queries;
use tokio;
use tokio_postgres::NoTls;

//serving frontend stuff
#[get("/images/favicon.ico")]
async fn serve_icon() -> impl Responder {
    return NamedFile::open_async("./client/images/favicon.ico").await;
}

#[get("/")]
async fn serve_home() -> impl Responder {
    return NamedFile::open_async("./client/index.html").await;
}

#[get("/chat")]
async fn serve_chat() -> impl Responder {
    return NamedFile::open_async("./client/chat.html").await;
}

#[get("/login")]
async fn serve_login() -> impl Responder {
    return NamedFile::open_async("./client/login.html").await;
}

#[get("/index.js")]
async fn serve_homejs() -> impl Responder {
    return NamedFile::open_async("./client/index.js").await;
}

#[get("/chat.js")]
async fn serve_chatjs() -> impl Responder {
    return NamedFile::open_async("./client/chat.js").await;
}

#[get("/login.js")]
async fn serve_loginjs() -> impl Responder {
    return NamedFile::open_async("./client/login.js").await;
}

#[get("/styles.css")]
async fn serve_styles() -> impl Responder {
    return NamedFile::open_async("./client/styles.css").await;
}
#[get("/index-styles.css")]
async fn serve_indexstyles() -> impl Responder {
    return NamedFile::open_async("./client/index-styles.css").await;
}
#[get("/chat-styles.css")]
async fn serve_chatstyles() -> impl Responder {
    return NamedFile::open_async("./client/chat-styles.css").await;
}
#[get("/login-styles.css")]
async fn serve_loginstyles() -> impl Responder {
    return NamedFile::open_async("./client/login-styles.css").await;
}

#[get("/styles.css.map")]
async fn serve_stylesmap() -> impl Responder {
    return NamedFile::open_async("./client/styles.css.map").await;
}
#[get("/index-styles.css.map")]
async fn serve_indexstylesmap() -> impl Responder {
    return NamedFile::open_async("./client/index-styles.css.map").await;
}
#[get("/chat-styles.css.map")]
async fn serve_chatstylesmap() -> impl Responder {
    return NamedFile::open_async("./client/chat-styles.css.map").await;
}
#[get("/login-styles.css.map")]
async fn serve_loginstylesmap() -> impl Responder {
    return NamedFile::open_async("./client/login-styles.css.map").await;
}

//queries/db stuff
#[get("/message/get/{number}/{starting}")]
async fn get_messages(data: web::Path<(i64, i32)>) -> impl Responder {
    let (number, starting_from) = data.into_inner();
    let (client, connection) = match tokio_postgres::connect(
        "host=localhost 
        dbname=chat_app 
        user=yabeni",
        NoTls,
    )
    .await
    {
        Ok(client) => client,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("error: {}", e);
        }
    });
    match queries::get_message_query(number, starting_from, client).await {
        Ok(response_vec) => {
            let mut html: String = String::new();
            response_vec.iter().for_each(|message| {
                let message_sender_display_color = match &message.sender.display_color {
                    Some(color) => color.clone(),
                    None => String::from("ffffff")
                };
                let message_sender_avatar_url = match &message.sender.avatar_url {
                    Some(url) => url.clone(),
                    None => String::from("none")
                };
                html.push_str(format!("
                            <div class='message-box'>
                                <div class='avatar' style='background-image: {message_sender_avatar}'></div>
                                <div class='message-content'>
                                    <div class='message-bar'>
                                        <h1 class='display-name' style='color: {message_sender_color}'>{message_sender_name}</h1>
                                        <p class='message-info'>usr_id: {message_sender_id} msg_id: {message_id}</p>
                                    </div>
                                    <p class='message-text'>{message_content}</p>
                                </div>
                            </div>
                        ", 
                        message_sender_avatar = message_sender_avatar_url, 
                        message_sender_color = message_sender_display_color, 
                        message_sender_name = message.sender.display_name, 
                        message_sender_id = message.sender.id, 
                        message_id = message.id, 
                        message_content = message.content)
                    .as_str());
            });
            let last_message =
                match response_vec.last() {
                    Some(last_message) => last_message,
                    None => return HttpResponse::InternalServerError().body(
                        "ERROR: message collection empty when trying to determine latest message",
                    ),
                };
            html.push_str(format!("<div id='top_of_chat' class='load-button' hx-trigger='click' hx-target='this' hx-swap='outerHTML' hx-get='/message/get/10/{}'>Load More</div>", last_message.id).as_str());
            return HttpResponse::Ok()
        }
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}

#[post("/message/post")]
async fn create_message(data: web::Form<models::IncomingMessage>) -> impl Responder {
    println!("received message post request {:#?}", data);
    let (client, connection) = match tokio_postgres::connect(
        "host=localhost 
        dbname=chat_app 
        user=yabeni",
        NoTls,
    )
    .await
    {
        Ok(client) => client,
        Err(err) => return HttpResponse::InternalServerError().body(format!("ERROR: error establishing database connection . {}", err)),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("ERROR: error establishing database connection . {}", e);
        }
    });

    match queries::create_message_query(data.into_inner(), client).await {
        Ok(_) => return HttpResponse::Ok().body("success"),
        Err(err) => return HttpResponse::InternalServerError().body(format!("ERROR: error querying database . {}", err)),
    };
}

#[post("/user/create")]
async fn create_user(user: web::Json<models::IncomingUser>) -> impl Responder {
    let (client, connection) = match tokio_postgres::connect(
        "host=localhost 
        dbname=chat_app 
        user=yabeni",
        NoTls,
    )
    .await
    {
        Ok(client) => client,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let db_response = match queries::create_user_query(user.into_inner(), client).await {
        Ok(rows_changed) => rows_changed,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    };

    return HttpResponse::Ok().body(format!("lol {}", db_response));
}

#[get("/user/{user_id}")]
async fn get_user(id: web::Path<i32>) -> impl Responder {
    let (client, connection) = match tokio_postgres::connect(
        "host=localhost 
        dbname=chat_app 
        user=yabeni",
        NoTls,
    )
    .await
    {
        Ok(client) => client,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let db_response = match queries::get_user_query(id.into_inner(), client).await {
        Ok(response) => response,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{}", err)),
    };
    return HttpResponse::Ok().body(format!("{:?}", db_response));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("started server at http://localhost:8080/");
    HttpServer::new(|| {
        App::new()
            .service(serve_home)
            .service(serve_icon)
            .service(serve_login)
            .service(serve_styles)
            .service(serve_homejs)
            .service(serve_chatjs)
            .service(serve_loginjs)
            .service(serve_styles)
            .service(serve_indexstyles)
            .service(serve_chatstyles)
            .service(serve_loginstyles)
            .service(serve_stylesmap)
            .service(serve_indexstylesmap)
            .service(serve_chatstylesmap)
            .service(serve_loginstylesmap)
            .service(serve_chat)
            .service(create_user)
            .service(get_user)
            .service(create_message)
            .service(get_messages)
    })
    .bind(("127.0.0.1", 4269))?
    .run()
    .await
}
