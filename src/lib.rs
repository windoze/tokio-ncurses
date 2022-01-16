use libc::{close, open, ttyname, O_RDONLY};
use tokio::{io::unix::AsyncFd, sync::OnceCell};

#[derive(Debug)]
struct TerminalFD(i32);

impl TerminalFD {
    fn new() -> Self {
        unsafe {
            let tn = ttyname(0);
            let fd = open(tn, O_RDONLY);
            if fd < 0 {
                panic!("Failed to open TTY.");
            }
            Self(fd)
        }
    }
}

impl Drop for TerminalFD {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe { close(self.0) };
        }
    }
}

static ONCE: OnceCell<TerminalFD> = OnceCell::const_new();

struct GetchFuture(AsyncFd<i32>);

impl std::future::Future for GetchFuture {
    type Output = i32;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.0.poll_read_ready(cx).is_ready() {
            std::task::Poll::Ready(ncurses::getch())
        } else {
            std::task::Poll::Pending
        }
    }
}

/// Async equivalent of `ncurses::getch()`
///
/// NOTE: Since ncurses functions are *not* thread-safe, you need to make sure this function and all other ncurses
/// functions are always called in the same thread.
///
/// Refer to the example to see how to use this function.
pub async fn getch() -> i32 {
    GetchFuture(AsyncFd::new(ONCE.get_or_init(|| async { TerminalFD::new() }).await.0).unwrap())
        .await
}
