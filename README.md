# CHIP-8 Rust
Basic CHIP-8 Emulator made in Rust

## Author:
Connor Byerman

## Sources:
General Guide: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/  
Another Rust Chip 8 Emulator: https://github.com/Rust-SDL2/rust-sdl2  
c8games taken from: https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html

## About:
This is a simple Chip 8 Emulator created in rust. It utilized multithreading to 
split the emulators in to two distinct processes defined by emulator functions:
`run_next_instr()` and `extract_timers()`. The former runs through a single 
instruction provided by the rom file, altering the Emulator struct as needed.

The latter simply extracts the references to the two timers stored in the object
(both of type `Arc<Mutex>`) so that they can be decremented in the parallel thread.
Said thread is scoped to avoid the timers' parent object being destroyed during.

## Keyboard Mapping:
This section on the keyboard:

| C1 | C2 | C3 | C4 |
| --- | --- | --- | --- |
| 1 | 2 | 3 | 4 |
| Q | W | E | R |
| A | S | D | F |
| Q | W | C | V |

Will map to:

| C1 | C2 | C3 | C4 |
| --- | --- | --- | --- |
| 1 | 2 | 3 | F |
| 4 | 5 | 6 | E |
| 7 | 8 | 9 | D |
| A | 0 | B | C |

## How To Run:

This program is fairly simple to run, provided you have the `cargo` and `rustc`
installed on your machine. Please note that this program has only been tested 
on Windows, further support is upcoming. Now the process to run it is simply 
running:  

`cargo run`  

within the project root directory and enter any other font you have installed on 
your machine. If not, don't worry, a default comes pre-packaged. After which you 
simply enter the name of the game you wish to play and the appropriate window will 
open.

## Some Games You can Play:

- PUZZLE <img width="942" height="471" alt="chip8-puzzle" src="https://github.com/user-attachments/assets/13fe05c2-22fc-4af4-ad3e-4c5450d7c63a" />

- BLITZ <img width="946" height="476" alt="chip8-blitz" src="https://github.com/user-attachments/assets/9372ccc7-78c9-479f-a6eb-0085d15ec93a" />

- BRIX <img width="956" height="484" alt="chip8-brix" src="https://github.com/user-attachments/assets/4ac50d5e-f512-479e-87a0-53853540469e" />

- PONG <img width="952" height="467" alt="chip8-pong" src="https://github.com/user-attachments/assets/576db20a-6c5b-4366-a53d-b6866b261969" />

- TETRIS <img width="941" height="480" alt="chip8-tetris" src="https://github.com/user-attachments/assets/b3329aac-2829-408e-8d45-d9544119455f" />
- INVADERS <img width="959" height="481" alt="chip8-invaders" src="https://github.com/user-attachments/assets/2fe12267-6e60-4020-be17-18865ccbd552" />

The rest of these can be found in the `test_games/c8games/` directory. Just type 
in whichever you wish to play.
