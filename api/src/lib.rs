use async_recursion::async_recursion;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Guess {
    #[serde(rename(deserialize = "cows"))]
    wrong_position: i32,
    #[serde(rename(deserialize = "bulls"))]
    right_position: i32,
    #[serde(rename(deserialize = "gameId"))]
    game_id: String,
    guesses: i32,
    solved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum StateCode {
    RUN,
    STOP,
    ERROR,
}

#[derive(Clone, Copy, Debug)]
struct State {
    pos_one: i32,
    pos_two: i32,
    pos_three: i32,
    code: StateCode,
    // guess: Guess,
}

/// A simple Spin HTTP component.
#[http_component]
async fn handle_api(_req: Request) -> anyhow::Result<impl IntoResponse> {
    let resp: Response = spin_sdk::http::send(Request::get(
        "https://bulls-n-cows.fermyon.app/api?guess=012",
    ))
    .await?;
    let resp = resp
        .into_builder()
        .header("spin-component", "rust-outbound-http")
        .build();
    // println!("RESP {resp:?}");
    let body = resp.body();
    let s = std::str::from_utf8(body).unwrap();
    // println!("RESP BODY STR {s:?}");

    let first_guess: Guess = serde_json::from_slice(body).unwrap();
    println!("First Guess: {first_guess:?}");

    let first_state = State {
        pos_one: 0,
        pos_two: 1,
        pos_three: 2,
        code: StateCode::RUN,
        // guess: first_guess.clone(),
    };
    println!("First State: {first_state:?}");

    let last_state = helper(first_guess, first_state).await;
    // let last_guess = last_state.guess;
    println!("Last State: {last_state:?}");
    // let res = serde_json::to_string(&last_guess);

    Ok(http::Response::builder()
        .status(StatusCode::OK)
        .body(s.to_string())?)
}

async fn call(game_id: String, state: State) -> Guess {
    println!("CALL {state:?}");
    let one = state.pos_one;
    let two = state.pos_two;
    let three = state.pos_three;
    let guess_string = format!("{one}{two}{three}");
    println!("CALL GUESS #: {guess_string:?}");
    let uri: String =
        format!("https://bulls-n-cows.fermyon.app/api?guess={guess_string}&id={game_id}");
    let resp: Response = spin_sdk::http::send(Request::get(uri)).await.unwrap();

    let resp = resp
        .into_builder()
        .header("spin-component", "rust-outbound-http")
        .build();
    let body = resp.body();
    // let s = std::str::from_utf8(body).unwrap();
    // println!("CALL RESP BODY STR {s:?}");

    let guess: Guess = serde_json::from_slice(body).unwrap();
    println!("CALL Guess TYPE: {guess:?}");

    if guess.wrong_position == 0 && guess.right_position == 3 && guess.solved {
        println!("!!! SOLUTION = {guess_string:?}!!!")
    }

    guess
}

async fn inc(guess: Guess, state: State) -> (Guess, State) {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
    } = state;
    println!("INC {state:?}");
    let game_id = guess.game_id.clone();

    let inc_max = vec![pos_one, pos_two, pos_three].iter().max().unwrap() + 1;

    let new_state_one = State {
        pos_one: inc_max,
        pos_two,
        pos_three,
        code,
    };
    let new_guess_one = call(game_id.clone(), new_state_one).await;

    let new_state_two = State {
        pos_one,
        pos_two: inc_max,
        pos_three,
        code,
    };
    let new_guess_two = call(game_id.clone(), new_state_two).await;

    let new_state_three = State {
        pos_one,
        pos_two,
        pos_three: inc_max,
        code,
    };
    let new_guess_three = call(game_id.clone(), new_state_three).await;

    let tuples = vec![
        (new_guess_one, new_state_one),
        (new_guess_two, new_state_two),
        (new_guess_three, new_state_three),
    ];

    let mut max = (guess.clone(), state.clone());
    println!("INC MAX BEFORE: {guess:?} {state:?}");

    for (g, s) in tuples {
        if g.right_position >= max.clone().0.right_position
            && g.wrong_position >= max.clone().0.wrong_position
        {
            max = (g, s);
        } else {
        }
    }
    println!("INC MAX AFTER: {guess:?} {state:?}");

    max
}

async fn swap(guess: Guess, state: State) -> (Guess, State) {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
    } = state;
    println!("SWAP {state:?}");
    let game_id = guess.game_id.clone();

    let new_state_one = State {
        pos_one: pos_two,
        pos_two: pos_one,
        pos_three: pos_three,
        code,
    };
    let new_guess_one = call(game_id.clone(), new_state_one).await;

    let new_state_two = State {
        pos_one: pos_two,
        pos_two: pos_three,
        pos_three: pos_one,
        code,
    };
    let new_guess_two = call(game_id.clone(), new_state_two).await;

    let new_state_three = State {
        pos_one: pos_three,
        pos_two: pos_one,
        pos_three: pos_two,
        code,
    };
    let new_guess_three = call(game_id.clone(), new_state_three).await;

    let tuples = vec![
        (new_guess_one, new_state_one),
        (new_guess_two, new_state_two),
        (new_guess_three, new_state_three),
    ];

    let mut max = (guess.clone(), state.clone());
    println!("SWAP MAX BEFORE: {guess:?} {state:?}");

    for (g, s) in tuples {
        if g.right_position >= max.clone().0.right_position
            && g.wrong_position >= max.clone().0.wrong_position
        {
            max = (g, s);
        } else {
        }
    }
    println!("SWAP MAX AFTER: {guess:?} {state:?}");

    max
}

#[async_recursion(?Send)]
async fn helper(guess: Guess, state: State) -> State {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
    } = state;
    if pos_one > 4 || pos_two > 4 || pos_three > 4 {
        return State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::ERROR,
        };
    }

    if code == StateCode::STOP {
        return state;
    }
    if code == StateCode::ERROR {
        return state;
    }

    match guess.clone() {
        Guess {
            wrong_position,
            right_position,
            game_id,
            guesses,
            solved,
        } if solved => State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::STOP,
        },
        Guess {
            wrong_position: 0,
            right_position: 0,
            game_id,
            guesses,
            solved,
        } => {
            let new_state = State {
                pos_one: pos_one + 1,
                pos_two: pos_two + 2,
                pos_three: pos_three + 3,
                code,
            };
            let new_guess = call(game_id.clone(), new_state).await;
            helper(new_guess, new_state).await
        }
        Guess {
            wrong_position: 0,
            right_position: 1,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }
        Guess {
            wrong_position: 0,
            right_position: 2,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 0,
            right_position: 3,
            game_id,
            guesses,
            solved,
        } => State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::STOP,
        },

        Guess {
            wrong_position: 1,
            right_position: 0,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = swap(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 1,
            right_position: 1,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 2,
            right_position: 0,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 2,
            right_position: 1,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = swap(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 3,
            right_position: 0,
            game_id,
            guesses,
            solved,
        } => {
            let (new_guess, new_state) = swap(guess, state).await;
            helper(new_guess, new_state).await
        }

        _ => State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::ERROR,
        },
    }
}
