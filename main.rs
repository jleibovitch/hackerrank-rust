use std::io::stdin;
use rand::Rng;

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

  let mcts = MCTS::new(board, player_id);
  mcts.start_search();

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

struct Node {

  explored_children: Vec<(usize, usize)>,
  unexplored_children: Vec<usize>,
  parent: Option<usize>,
  node_index: usize,
  state: Board,
  wins: i32,
  losses: i32,
  visits: u32
}

impl Node {

  pub fn new(parent: Option<usize>, state: Board, node_index: usize) -> Self {
    Node{
      unexplored_children: state.get_valid_moves(),
      parent,
      state,
      node_index,
      explored_children: Vec::new(),
      visits: 0,
      losses: 0,
      wins: 0,
    }
  }

  fn best_uct(&self, nodes: &Vec<Box<Node>>) -> usize {
    let current_node = &nodes[self.node_index];
    let children = current_node.explored_children.to_vec();

    let ucts: Vec<(usize, f64)> = children.into_iter().map(|x| {
      let node = &nodes[x.0];
      (x.0, (node.wins - node.losses) as f64 / node.visits as f64 + (2_f64).sqrt() * ((current_node.visits as f64).ln() / node.visits as f64).sqrt())
    }).collect();

    let mut best_node = 0;
    let mut best_val: f64 = -10000_f64;
    for i in 0..ucts.len() {
      if ucts[i].1 > best_val {
        best_node = ucts[i].0;
        best_val = ucts[i].1;
      }
    }

    best_node
  }

  fn expand(&mut self) -> Option<(Board, usize)> {
    let next_move = self.unexplored_children.pop()?;
    Some((self.state.update_board(next_move), next_move))
  }
}

// #[derive(Default)]
struct Tree {
  nodes: Vec<Box<Node>>
}

impl Tree {

  pub fn new(root_state: Board) -> Self {
    let root = Box::new(Node::new(None, root_state, 0));
    Tree{
      nodes: vec![root]
    }
  }

  fn add_child(&mut self, state: Board, parent: Option<usize>) -> usize {

    let node_index = self.nodes.len();
    let node = Box::new(Node::new(parent, state, node_index));
    self.nodes.push(node);

    node_index

  }
}

struct MCTS {
  tree: Tree,
  current_player: u32
}

impl MCTS {
  pub fn new(root_state: Board, current_player: u32) -> Self {

    MCTS{
      tree: Tree::new(root_state),
      current_player,
    }

  }

  fn start_search(mut self) {

    for _ in 0..100000 {
      let leaf = self.traverse();
      let result = self.simulate(leaf);
      self.backpropogate(leaf, result);
    }
    
    let (mut best_child, mut max_visits) = (0,0);
    // println!("{:?}", self.tree.nodes[0].explored_children);
    for (child_index, move_index) in self.tree.nodes[0].explored_children.to_vec().into_iter() {
      if self.tree.nodes[child_index].visits > max_visits {
        best_child = move_index;
        max_visits = self.tree.nodes[child_index].visits;
      }
    }

    println!("{}", best_child+1);
  }

  fn traverse(&mut self) -> usize {

    let mut current_node = 0;



    while self.tree.nodes[current_node].unexplored_children.len() == 0 {

      if self.tree.nodes[current_node].explored_children.len() == 0 {
        return current_node
      }

      current_node = self.tree.nodes[current_node].best_uct(&self.tree.nodes);
    }

    let (next_state, move_index) = self.tree.nodes[current_node].expand().unwrap();
    let child_index = self.tree.add_child(next_state, Some(current_node));
    self.tree.nodes[current_node].explored_children.push((child_index, move_index));
    child_index

  }

  fn simulate(&self, leaf: usize) -> bool {

    let mut current_state = self.tree.nodes[leaf].state.clone();
    let mut rng = rand::thread_rng();

    let (mut game_over, mut winner) = current_state.game_over();
    while !game_over {
      let valid_moves = current_state.get_valid_moves();
      let next_board = current_state.update_board(valid_moves[rng.gen_range(0, valid_moves.len())]);
      let results = next_board.game_over();
      game_over = results.0;
      winner = results.1;
      current_state = next_board
    }

    winner == self.current_player

  }

  fn backpropogate(&mut self, leaf: usize, result: bool) {

    let mut current_node = Some(leaf);

    while current_node.is_some() {
      let node = &mut self.tree.nodes[current_node.unwrap()];
      node.visits+=1;
      if result {
        node.wins += 1;
      } else {
        node.losses += 1;
      }

      current_node = node.parent;
    }

  }

}