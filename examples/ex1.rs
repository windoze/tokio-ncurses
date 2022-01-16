use std::time::Duration;

use ncurses::*;
use tokio::sync::mpsc::channel;

// ncurses functions are *not* thread-safe
#[tokio::main(flavor = "current_thread")]
async fn main() {
    /* Start ncurses. */
    initscr();

    /* Print to the back buffer. */
    addstr("Hello, world!\n");
    addstr("Press Space Bar or wait 10 seconds to exit.\n");

    /* Update the screen. */
    refresh();

    /* Create additional runtime if you need to run background tasks,
     * and use channel to communicate with them.
     * Make sure all UI operations happen in the main thread/runtime.
     */
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    /* Create communication channel */
    let (sender, mut receiver) = channel(100);

    /* Spawn background task in additional runtime */
    runtime.spawn(async move {
        for n in 0..10 {
            if sender.send(10 - n).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    /* Wait for 10 seconds or Space Bar then exit */
    loop {
        tokio::select! {
            number = receiver.recv() => {
                match number {
                    Some(n) => {
                        addstr(&format!("Counting down {}\n", n));
                    },
                    None => {
                        /* This message is barely visible */
                        addstr("Sender stopped\n");
                        break;
                    }
                }
                refresh();
            }
            key = tokio_ncurses::getch() => {
                if key == 32 {
                    break;
                }
            }
        }
    }

    /* Terminate ncurses. */
    endwin();

    /* Shutdown additional runtime asynchronously */
    runtime.shutdown_background();
}
