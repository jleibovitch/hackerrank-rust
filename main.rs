use std::io::stdin;

fn main() {
  let mut input_buffer = String::new();

  stdin().read_line(&mut input_buffer).unwrap();
  stdin().read_line(&mut input_buffer).unwrap();
  stdin().read_line(&mut input_buffer).unwrap();
  stdin().read_line(&mut input_buffer).unwrap();
  stdin().read_line(&mut input_buffer).unwrap();

  let inputs: Vec<&str> = input_buffer.trim().splitn(5, '\n').collect();

  let player_1_marbles = inputs[2]
    .split_whitespace()
    .filter_map(|s: &str| s.parse().ok())
    .collect();
  let player_2_marbles = inputs[4]
    .split_whitespace()
    .filter_map(|s: &str| s.parse().ok())
    .collect();

  get_next_move(
    inputs[0].parse().unwrap(),
    inputs[1].parse().unwrap(),
    player_1_marbles,
    inputs[3].parse().unwrap(),
    player_2_marbles,
  );
}

fn get_next_move(
  player_id: u32,
  player_1_mancala: u32,
  player_1_marbles: Vec<u32>,
  player_2_mancala: u32,
  player_2_marbles: Vec<u32>,
) {
  let board = Board {
    current_player: player_id,
    player_1_board: player_1_marbles,
    player_1_score: player_1_mancala,
    player_2_board: player_2_marbles,
    player_2_score: player_2_mancala,
  };
}

#[derive(Clone)]
struct Board {
  player_1_board: Vec<u32>,
  player_2_board: Vec<u32>,
  player_1_score: u32,
  player_2_score: u32,
  current_player: u32,
}

impl Board {
  fn update_board(&self, move_index: usize) -> Self {
    let (mut current_board, mut current_score, mut opponent_board) = if self.current_player == 1 {
      (
        self.player_1_board.to_vec(),
        self.player_1_score,
        self.player_2_board.to_vec(),
      )
    } else {
      (
        self.player_2_board.to_vec(),
        self.player_2_score,
        self.player_1_board.to_vec(),
      )
    };

    let mut next_player = if self.current_player == 1 { 2 } else { 1 };

    let mut current_index = move_index;
    let num_marbles = current_board[current_index];
    current_board[current_index] = 0;

    for _ in 0..num_marbles {
      current_index += 1;
      current_index %= 13;

      if current_index < 6 {
        current_board[current_index] += 1;
      } else if current_index == 6 {
        current_score += 1
      } else {
        opponent_board[current_index - 7] += 1
      }
    }
    // update behaviour depending on last location
    if current_index == 6 {
      next_player = self.current_player
    } else if current_index < 6 && current_board[current_index] == 1 {
      current_board[current_index] += opponent_board[5 - current_index];
      opponent_board[5 - current_index] = 0;
    }

    if self.current_player == 1 {
      Board {
        current_player: next_player,
        player_1_board: current_board,
        player_1_score: current_score,
        player_2_board: opponent_board,
        player_2_score: self.player_2_score,
      }
    } else {
      Board {
        current_player: next_player,
        player_1_board: opponent_board,
        player_1_score: self.player_1_score,
        player_2_board: current_board,
        player_2_score: current_score,
      }
    }
  }

  fn get_valid_moves(&self) -> Vec<usize> {
    let current_board = if self.current_player == 1 {
      &self.player_1_board
    } else {
      &self.player_2_board
    };
    let mut valid_moves = Vec::new();

    for i in 0..6 {
      if current_board[i] > 0 {
        valid_moves.push(i);
      }
    }

    valid_moves
  }

  fn game_over(&self) -> (bool, u32) {
    let player_1: u32 = self.player_1_board.to_vec().into_iter().sum();
    let player_2: u32 = self.player_2_board.to_vec().into_iter().sum();

    if player_1 == 0 || player_2 == 0 {
      let winner = if player_1 + self.player_1_score > player_2 + self.player_2_score {
        1
      } else {
        2
      };
      (true, winner)
    } else {
      (false, 0)
    }
  }
}
