use tokio::sync::broadcast;

pub struct SingleRequestShutdown {
    is_terminating: bool,
    notification: broadcast::Receiver<()>
}

impl SingleRequestShutdown {
    pub fn new(notify_rx: broadcast::Receiver<()>) -> Self
    {
        Self{notification:notify_rx, is_terminating:false}
    }

    pub fn is_shutdown(&self) -> bool { self.is_terminating }

    pub async fn recv(&mut self) {
        if !self.is_terminating {
            let _ = self.notification.recv().await;
            self.is_terminating = true;
        }
    }
} // end of SingleRequestShutdown
