use crate::service::todo::{GetTodoRequest, TodoItem};
use crate::{db::Message, service::todo::todo_server::Todo};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

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
        _request: Request<GetTodoRequest>,
    ) -> Result<Response<Self::GetTodosStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<TodoItem, Status>>(4);
        tokio::spawn(async move {
            tx.send(Ok(TodoItem {
                id: String::from("1"),
            }))
            .await
            .unwrap();
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
