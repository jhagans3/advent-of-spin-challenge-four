use std::collections::HashMap;

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

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
enum StateCode {
    RUN,
    STOP,
    ERROR,
}

#[derive(Clone, Serialize, Debug)]
struct State {
    #[serde(skip_serializing)]
    pos_one: i32,
    #[serde(skip_serializing)]
    pos_two: i32,
    #[serde(skip_serializing)]
    pos_three: i32,
    #[serde(skip_serializing)]
    code: StateCode,
    ans: i32,
    #[serde(rename(serialize = "total_backtrack_guesses"))]
    guesses: i32,
    game_id: String,
    history: HashMap<i32, String>,
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
    // let s = std::str::from_utf8(body).unwrap();
    // println!("RESP BODY STR {s:?}");

    let first_guess: Guess = serde_json::from_slice(body).unwrap();
    let w = first_guess.wrong_position;
    let r = first_guess.right_position;
    let first_guess_history = format!("012({w},{r})[1st]");
    println!("First Guess: {first_guess:?}");
    let mut history = HashMap::new();
    history.insert(0, "guess(cow,bull)[op]".to_string());
    history.insert(1, first_guess_history);

    let first_state = State {
        pos_one: 0,
        pos_two: 1,
        pos_three: 2,
        code: StateCode::RUN,
        ans: 000,
        guesses: 1,
        game_id: first_guess.game_id.clone(),
        history,
    };
    println!("First State: {first_state:?}");

    let last_state = helper(first_guess.clone(), first_state).await;
    println!("Last State: {last_state:?}");
    let res = serde_json::to_string(&last_state).unwrap();

    Ok(http::Response::builder().status(StatusCode::OK).body(res)?)
}

async fn call(state: State) -> Guess {
    println!("CALL {state:?}");
    let game_id = state.game_id;
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

fn found_answer(guess: &Guess) -> bool {
    if guess.wrong_position == 0 && guess.right_position == 3 && guess.solved {
        true
    } else {
        false
    }
}

fn new_state_res(state: State, op: String) -> State {
    let game_id = state.game_id.clone();
    let number_of_guess = state.guesses.clone();
    let one = state.pos_one;
    let two = state.pos_two;
    let three = state.pos_three;
    let guess_string = format!("{one}{two}{three}(0,3)[{op}]");
    let ans = (100 * one) + (10 * two) + (three);

    let mut history = state.history.clone();
    let key = history.len() as i32;
    history.insert(key, guess_string);

    State {
        pos_one: one,
        pos_two: two,
        pos_three: three,
        code: StateCode::STOP,
        ans: ans,
        guesses: number_of_guess,
        game_id,
        history,
    }
}

// fn update_history(state: &State) -> State {}

async fn inc(guess: Guess, state: State) -> (Guess, State) {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
        ans,
        guesses,
        game_id,
        history,
    } = state.clone();
    println!("INC {state:?}");
    let operation = "+".to_string();
    let inc_max = vec![pos_one, pos_two, pos_three].iter().max().unwrap() + 1;
    let key = history.len() as i32;

    let mut new_state_one = State {
        pos_one: inc_max,
        pos_two,
        pos_three,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history: state.history.clone(),
    };

    let new_guess_one = call(new_state_one.clone()).await;
    if found_answer(&new_guess_one) {
        let new_state_res = new_state_res(new_state_one, operation);
        println!("!!! INCSOLUTION = {new_guess_one:?} {new_state_res:?}!!!");
        return (new_guess_one, new_state_res.clone());
    } else {
        let w = new_guess_one.wrong_position.clone();
        let r = new_guess_one.right_position.clone();
        let mut history_one = new_state_one.history.clone();
        let guess_string = format!("{inc_max}{pos_two}{pos_three}({w},{r})[inc]");
        history_one.insert(key, guess_string);
        new_state_one.history = history_one;
    }

    let mut new_state_two = State {
        pos_one,
        pos_two: inc_max,
        pos_three,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history: state.history.clone(),
    };

    let new_guess_two = call(new_state_two.clone()).await;
    if found_answer(&new_guess_two) {
        let new_state_res = new_state_res(new_state_two, operation);
        println!("!!! INCSOLUTION = {new_guess_two:?} {new_state_res:?}!!!");
        return (new_guess_two, new_state_res.clone());
    } else {
        let w = new_guess_two.wrong_position.clone();
        let r = new_guess_two.right_position.clone();
        let mut history_two = new_state_two.history.clone();
        let guess_string = format!("{pos_one}{inc_max}{pos_three}({w},{r})[inc]");
        history_two.insert(key, guess_string);
        new_state_two.history = history_two;
    }

    let mut new_state_three = State {
        pos_one,
        pos_two,
        pos_three: inc_max,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history: state.history.clone(),
    };

    let new_guess_three = call(new_state_three.clone()).await;
    if found_answer(&new_guess_three) {
        let new_state_res = new_state_res(new_state_three, operation);
        println!("!!! INCSOLUTION = {new_guess_three:?} {new_state_res:?}!!!");
        return (new_guess_three, new_state_res.clone());
    } else {
        let w = new_guess_three.wrong_position.clone();
        let r = new_guess_three.right_position.clone();
        let mut history_three = new_state_three.history.clone();
        let guess_string = format!("{pos_one}{pos_two}{inc_max}({w},{r})[inc]");
        history_three.insert(key, guess_string);
        new_state_three.history = history_three;
    }

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
    let (max_guess, max_state) = max.clone();
    println!("INC MAX AFTER: {max_guess:?} {max_state:?}");

    max
}

async fn swap(guess: Guess, state: State) -> (Guess, State) {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
        ans,
        guesses,
        game_id,
        history,
    } = state.clone();
    let operation = "~".to_string();
    println!("SWAP {state:?}");

    let key = history.len() as i32;

    let mut new_state_one = State {
        pos_one: pos_two,
        pos_two: pos_one,
        pos_three: pos_three,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history,
    };

    let new_guess_one = call(new_state_one.clone()).await;
    if found_answer(&new_guess_one) {
        let new_state_res = new_state_res(new_state_one, operation);
        println!("!!! SWAPSOLUTION = {new_guess_one:?} {new_state_res:?}!!!");
        return (new_guess_one, new_state_res.clone());
    } else {
        let w = new_guess_one.wrong_position.clone();
        let r = new_guess_one.right_position.clone();
        let mut history_one = new_state_one.history.clone();
        let history_string = format!("{pos_two}{pos_one}{pos_three}({w},{r})[swp]");
        history_one.insert(key, history_string);
        new_state_one.history = history_one;
    }

    let mut new_state_two = State {
        pos_one: pos_two,
        pos_two: pos_three,
        pos_three: pos_one,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history: state.history.clone(),
    };

    let new_guess_two = call(new_state_two.clone()).await;
    if found_answer(&new_guess_two) {
        let new_state_res = new_state_res(new_state_two, operation);
        println!("!!! SWAPSOLUTION = {new_guess_two:?} {new_state_res:?}!!!");
        return (new_guess_two, new_state_res.clone());
    } else {
        let w = new_guess_two.wrong_position.clone();
        let r = new_guess_two.right_position.clone();
        let mut history_two = new_state_two.history.clone();
        let history_string = format!("{pos_two}{pos_three}{pos_one}({w},{r})[swp]");
        history_two.insert(key, history_string);
        new_state_two.history = history_two;
    }

    let mut new_state_three = State {
        pos_one: pos_three,
        pos_two: pos_one,
        pos_three: pos_two,
        code,
        ans,
        guesses,
        game_id: game_id.clone(),
        history: state.history.clone(),
    };

    let new_guess_three = call(new_state_three.clone()).await;
    if found_answer(&new_guess_three) {
        let new_state_res = new_state_res(new_state_three, operation);
        println!("!!! SWAPSOLUTION = {new_guess_three:?} {new_state_res:?}!!!");
        return (new_guess_three, new_state_res.clone());
    } else {
        let w = new_guess_three.wrong_position.clone();
        let r = new_guess_three.right_position.clone();
        let mut history_three = new_state_two.history.clone();
        let history_string = format!("{pos_three}{pos_one}{pos_two}({w},{r})[swp]");
        history_three.insert(key, history_string);
        new_state_three.history = history_three;
    }

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
    let (max_guess, max_state) = max.clone();
    println!("SWAP MAX AFTER: {max_guess:?} {max_state:?}");

    max
}

#[async_recursion(?Send)]
async fn helper(guess: Guess, state: State) -> State {
    let State {
        pos_one,
        pos_two,
        pos_three,
        code,
        ans,
        guesses,
        game_id,
        history,
    } = state.clone();

    if pos_one > 4 || pos_two > 4 || pos_three > 4 {
        return State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::ERROR,
            ans,
            guesses,
            game_id,
            history,
        };
    }

    match guess.clone() {
        Guess {
            wrong_position: _,
            right_position: _,
            game_id: _,
            guesses,
            solved,
        } if solved => State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::STOP,
            ans,
            guesses,
            game_id,
            history,
        },
        Guess {
            wrong_position: 0,
            right_position: 0,
            game_id,
            guesses,
            solved: _,
        } => {
            let new_state = State {
                pos_one: pos_one + 1,
                pos_two: pos_two + 2,
                pos_three: pos_three + 3,
                code,
                ans,
                guesses,
                game_id: game_id.clone(),
                history,
            };
            let new_guess = call(new_state.clone()).await;
            helper(new_guess, new_state).await
        }
        Guess {
            wrong_position: 0,
            right_position: 1,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }
        Guess {
            wrong_position: 0,
            right_position: 2,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 0,
            right_position: 3,
            game_id: _,
            guesses,
            solved: _,
        } => {
            println!("03 {state:?} {guess:?}");
            State {
                pos_one,
                pos_two,
                pos_three,
                code: StateCode::STOP,
                ans,
                guesses,
                game_id,
                history,
            }
        }

        Guess {
            wrong_position: 1,
            right_position: 0,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 1,
            right_position: 1,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 2,
            right_position: 0,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = inc(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 2,
            right_position: 1,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = swap(guess, state).await;
            helper(new_guess, new_state).await
        }

        Guess {
            wrong_position: 3,
            right_position: 0,
            game_id: _,
            guesses: _,
            solved: _,
        } => {
            let (new_guess, new_state) = swap(guess, state).await;
            helper(new_guess, new_state).await
        }

        _ => State {
            pos_one,
            pos_two,
            pos_three,
            code: StateCode::ERROR,
            ans,
            guesses,
            game_id,
            history,
        },
    }
}
