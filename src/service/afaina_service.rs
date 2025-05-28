use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use futures::{pin_mut, SinkExt, Stream, StreamExt};
use ollama_rs::generation::chat::{ChatMessage, ChatMessageResponseStream};
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::Ollama;

use tonic::{Code, Request, Response, Status, Streaming};
use tonic::codegen::http::request;
use crate::afaina::afaina_service_server::AfainaService;
use crate::afaina::{chat_request, chat_response, AfainaMessage, ChatRequest, ChatResponse};

pub struct Afaina{
    model:String,
}

impl Afaina {
    pub fn new(model:String)->Afaina{
            Afaina{
                model,
            }
    }
}

#[tonic::async_trait]
impl  AfainaService for Afaina {

    type ChatStream = Pin<Box<dyn Stream<Item = Result<ChatResponse, Status>> + Send>>;

    async fn chat(&self, request: Request<Streaming<ChatRequest>>) -> Result<Response<Self::ChatStream>, Status> {

            let ollama = Ollama::default();
            let history = Arc::new(Mutex::new(vec![]));

            let mut stream = request.into_inner();
            let(tx, mut rx) =  tokio::sync::mpsc::channel::<String>(32);

            tokio::spawn(async move {
                while let Some(chat_request) = stream.next().await {
                    match chat_request {
                        Ok(req) => {
                            if let Some(chat_request::Request::Msg(msg)) = req.request{
                                    if tx.send(msg.message).await.is_err(){
                                        break;
                                    }
                            }
                        },
                        Err(err) => {
                            println!("AfainaService Receive Error: {:?}", err);
                            break;
                        }
                    }
                }
            });

            let f_msg = match rx.recv().await {
                Some(msg) => msg,
                None => return Err(Status::new(Code::Internal,"internal error while processing stream")),
            };

            let mut chat_response_stream = ollama
                .send_chat_messages_with_history_stream(
                    history.clone(),
                    ChatMessageRequest::new(
                        self.model.clone(),
                        vec![ChatMessage::user(f_msg)],
                    )
                ).await.map_err(|e| Status::new(Code::Internal,format!("error while sending chat messages: {}", e)));

            let output =  async_stream::try_stream!{
                 let mut chat_stream = chat_response_stream.map_err(|e| Status::internal(e.to_string()))?;
                 pin_mut!(chat_stream);

                while let Some(msg) = chat_stream.next().await {
                    let res = msg.map_err(|e| Status::internal("response result err"))?;

                    yield ChatResponse{
                        response: Some(chat_response::Response::Msg(AfainaMessage{
                            response_message: res.message.content,
                        })),
                    }
                }
            };

            dbg!(history.lock().unwrap().clone());

            Ok(Response::new(Box::pin(output)))
        

    }
}