use std::{pin::Pin, task::{Poll, Context}, sync::Arc};

use futures::{future::poll_fn, ready};
use hyper::server::{conn::{AddrIncoming, AddrStream}, accept::Accept as HyperAccept};
use rustls::{server::ClientHello, ServerConfig};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel, UnboundedSender};
use tokio_rustls::{server::TlsStream, LazyConfigAcceptor};

pub struct TlsAcceptor {
    receiver: UnboundedReceiver<TlsStream<AddrStream>>,
}

#[async_trait::async_trait]
pub trait ResolvesServerConf {
    async fn resolve_server_config(self: Arc<Self>, client_hello: &ClientHello) -> Option<Arc<ServerConfig>>;
}

impl TlsAcceptor {
    pub fn new <R: ResolvesServerConf + Send + Sync + 'static> (incoming: AddrIncoming, resolver: Arc<R>) -> TlsAcceptor {
        let (sender, receiver) = unbounded_channel::<TlsStream<AddrStream>>();
        tokio::task::spawn(Self::accept_loop(incoming, resolver, sender));

        TlsAcceptor {
            receiver
        }
    }

    async fn accept_loop <R: ResolvesServerConf + Send + Sync + 'static> (mut incoming: AddrIncoming, resolver: Arc<R>, sender: UnboundedSender<TlsStream<AddrStream>>) {
        loop {
            match poll_fn(|ctx| Pin::new(&mut incoming).poll_accept(ctx)).await {
                Some(Ok(stream)) => {
                    tokio::task::spawn(Self::handle_stream(stream, resolver.clone(), sender.clone()));
                },
                Some(Err(e)) => eprintln!("tls_accceptor: error accepting incoming: {}", e),
                None => println!("tls_accceptor: incoming stream closed"),
            }
        }
    }

    async fn handle_stream <R: ResolvesServerConf + Send + Sync + 'static> (stream: AddrStream, resolver: Arc<R>, sender: UnboundedSender<TlsStream<AddrStream>>) {
        let acceptor = rustls::server::Acceptor::default();
        let tls_stream = match LazyConfigAcceptor::new(acceptor, stream).await {
            Err(err) => return eprintln!("tls_acceptor: accept error: {}", err),
            Ok(handshake) => match resolver.resolve_server_config(&handshake.client_hello()).await {
                Some(config) => match handshake.into_stream(config).await {
                    Ok(stream) => stream,
                    Err(err) => return eprintln!("tls_acceptor: handshake error: {}", err)
                }
                None => return eprintln!("tls_acceptor: no server config for client"),
            }
        };

        if let Err(e) = sender.send(tls_stream) {
            eprintln!("tls_accceptor: error sending result: {}", e);
        }
    }
}

impl HyperAccept for TlsAcceptor {
    type Conn = TlsStream<AddrStream>;
    type Error = std::io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(self.get_mut().receiver.poll_recv(cx)) {
            Some(stream) => Poll::Ready(Some(Ok(stream))),
            _ => Poll::Pending,
        }
    }
}