use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    marker::PhantomData,
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckId(pub u64);
impl Default for AckId {
    fn default() -> Self {
        Self(thread_rng().gen())
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Message<MessageContent> {
    AckReply(AckId),
    Ack((AckId, MessageContent)),
    NoAck(MessageContent),
}

pub struct AckUdpSocket<TxMessageContent, RxMessageContent>
where
    TxMessageContent: Serialize + for<'de> Deserialize<'de> + Debug,
    RxMessageContent: Serialize + for<'de> Deserialize<'de> + Debug,
{
    udp_socket: UdpSocket,
    resend_interval: std::time::Duration,
    messages: Vec<(Message<TxMessageContent>, SocketAddr, Option<SystemTime>)>,
    marker: PhantomData<RxMessageContent>,
}

impl<TxMessageContent, RxMessageContent> AckUdpSocket<TxMessageContent, RxMessageContent>
where
    TxMessageContent: Serialize + for<'de> Deserialize<'de> + Debug,
    RxMessageContent: Serialize + for<'de> Deserialize<'de> + Debug,
{
    pub fn new(udp_socket: UdpSocket, resend_interval: std::time::Duration) -> Self {
        Self {
            udp_socket,
            resend_interval,
            messages: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.udp_socket.local_addr()
    }

    pub fn queue(&mut self, message_content: TxMessageContent, addr: &SocketAddr, needs_ack: bool) {
        let message = match needs_ack {
            true => Message::Ack((AckId::default(), message_content)),
            false => Message::NoAck(message_content),
        };
        self.messages.push((message, *addr, None));
    }

    pub fn send_to(
        &mut self,
        message_content: TxMessageContent,
        addr: &SocketAddr,
        needs_ack: bool,
    ) {
        let message = match needs_ack {
            true => Message::Ack((AckId::default(), message_content)),
            false => Message::NoAck(message_content),
        };
        Self::send_single(&self.udp_socket, &message, addr);
        if needs_ack {
            self.messages
                .push((message, *addr, Some(SystemTime::now())));
        }
    }

    fn send_single(udp_socket: &UdpSocket, message: &Message<TxMessageContent>, addr: &SocketAddr) {
        let buf = rmp_serde::to_vec(&message).unwrap();
        udp_socket.send_to(buf.as_slice(), *addr).unwrap();
    }

    pub fn send_queued(&mut self) {
        self.messages
            .retain_mut(|(message, addr, last_sendt_time)| {
                if last_sendt_time
                    .is_some_and(|time| time.elapsed().unwrap() < self.resend_interval)
                {
                    return true;
                }
                *last_sendt_time = Some(SystemTime::now());
                Self::send_single(&self.udp_socket, message, addr);
                matches!(message, Message::Ack(_))
            });
    }

    pub fn receive(&mut self) -> Option<(RxMessageContent, SocketAddr)> {
        let mut buf = [0; 20000];
        let received_message = self.udp_socket.recv_from(&mut buf);
        let (bytes_received, addr) = match received_message {
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::ConnectionReset
                ) =>
            {
                return None
            }
            Err(e) => {
                dbg!(e);
                debug_assert!(false);
                return None;
            }
            Ok(x) => x,
        };
        let sized_buf = &buf[..bytes_received];
        let deserialization_result = rmp_serde::from_slice::<Message<RxMessageContent>>(sized_buf);
        let Ok(message) = deserialization_result else {
            dbg!(deserialization_result.err());
            debug_assert!(false);
            return None;
        };
        match message {
            Message::AckReply(ack_id) => {
                self.messages.retain_mut(
                    |(message, _, _)| matches!(message, Message::Ack((id, _)) if *id != ack_id),
                );
                self.receive()
            }
            Message::Ack((ack_id, content)) => {
                Self::send_single(&self.udp_socket, &Message::AckReply(ack_id), &addr);
                Some((content, addr))
            }
            Message::NoAck(content) => Some((content, addr)),
        }
    }
}
