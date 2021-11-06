use crate::interceptors::AuthExtension;
use crate::service::todo::{GetTodoRequest, TodoItem};
use crate::{db::Message, service::todo::todo_server::Todo};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::log::error;
#[derive(Debug)]
pub struct TodoService {
    db_message_sender: Sender<Message>,
}

impl TodoService {
    pub fn new(db_message_sender: Sender<Message>) -> Self {
        Self { db_message_sender }
    }
}

#[tonic::async_trait]
impl Todo for TodoService {
    type GetTodosStream = ReceiverStream<Result<TodoItem, Status>>;

    async fn get_todos(
        &self,
        request: Request<GetTodoRequest>,
    ) -> Result<Response<Self::GetTodosStream>, Status> {
        if let Some(auth_extensions) = request.extensions().get::<AuthExtension>() {
            let (tx, rx) = mpsc::channel::<Result<TodoItem, Status>>(4);
            let (db_tx, mut db_rx) = mpsc::channel::<Result<TodoItem, String>>(4);
            match self
                .db_message_sender
                .send(Message::GetTodos {
                    username: auth_extensions.username.to_string(),
                    resp: db_tx,
                })
                .await
            {
                Ok(_) => {}
                Err(e) => error!("Failed to send get todos message to DB manager {:?}", e),
            }
            tokio::spawn(async move {
                while let Some(message) = db_rx.recv().await {
                    match message {
                        Ok(res) => {
                            tx.send(Ok(res)).await.unwrap();
                        }
                        Err(e) => {
                            error!("Error while getting todos {:?}", e);
                            // Err(Status::aborted("Error while signing up"));
                        }
                    }
                }
            });
            Ok(Response::new(ReceiverStream::new(rx)))
        } else {
            return Err(Status::unauthenticated("Unauthorized request"));
        }
    }
}
