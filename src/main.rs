use rand::seq::SliceRandom;
use rand::Rng;

// Total number of games
const NUM_GAMES: usize = 1;

// Number of cards in each suit: 2-10, J, Q, K and A
const NUM_KC: usize = 13;

// Number of all cards
const NUM_CARDS: usize = NUM_KC * 4;

// Number of players; HEARTS expects to be played by four players.
const NUM_PLAYERS: usize = 4;

const CLUB: i32 = 0;
const DIA: i32 = 1;
const SPADE: i32 = 2;
const HEART: i32 = 3;

const C_2: i32 = 0;
const S_Q: i32 = SPADE * (NUM_KC as i32) + 10;

fn main() {
    // Assigning agents:
    // 1 -> Random agent; it plays cards from its hand at random.
    // 2 -> Rule-based agent; it plays cards based on the pre-determined rules.
    let idx: [i32; NUM_PLAYERS] = [1, 2, 2, 2];

    // Making instances of 4 agents and store the objects in Vec.
    let mut agents: Vec<Box<dyn Agent>> = Vec::new();
    for i in 0..NUM_PLAYERS {
        match idx[i] {
            1 => agents.push(Box::new(RandomAgent::new())),
            2 => agents.push(Box::new(RuleBasedAgent::new())),
            _ => panic!("occurred error1")
        }
    }

    let mut total_penalty_points: [f32; NUM_PLAYERS] = [0.0; NUM_PLAYERS];
    let mut averaged_penalty_points: [f32; NUM_PLAYERS] = [0.0; NUM_PLAYERS];

    // Letting agents play the card game "Hearts" NUM_GAMES times.
    for _ in 1..=NUM_GAMES {
        let mut whole_card_sequence: [i32; NUM_CARDS] = [-1; NUM_CARDS];
        let mut whole_agent_sequence: [i32; NUM_CARDS] = [-1; NUM_CARDS];

        play_one_game(
            &mut agents,
            &mut whole_card_sequence,
            &mut whole_agent_sequence,
        );

        let penalty_points = calc_penalty_points(&whole_card_sequence, &whole_agent_sequence);

        for i in 0..NUM_PLAYERS {
            total_penalty_points[i] += penalty_points[i] as f32;
        }
    }

    for i in 0..NUM_PLAYERS {
        averaged_penalty_points[i] = total_penalty_points[i] / (NUM_GAMES as f32);
    }
    println!("{:?}", averaged_penalty_points);
}

// fn play_one_game<T: Agent>(agents: &mut Vec<T>, whole_card_sequence: &mut [i32; NUM_CARDS], whole_agent_sequence: &mut [i32; NUM_CARDS]) {
fn play_one_game(agents: &mut Vec<Box<dyn Agent>>,
                 whole_card_sequence: &mut [i32; NUM_CARDS],
                 whole_agent_sequence: &mut [i32; NUM_CARDS]) {
    // Cards are dealt to the four agents so that each has NUM_KC cards at the beginning of a game.
    let dealt_cards = deal_cards(agents);
    // let ss = dealt_cards.iter()
    //     .fold(String::new(), |r, n| {
    //
    //         format!("{},{}", r, n.to_string() )
    //
    //     }).collect::<Vec<_>>();

    //let ss = dealt_cards.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(",");

    // Getting the playing sequence in the first trick based on agents' hands.
    // (the agent who has C-2 is the leading player in the initial trick).
    // let idx = dealt_cards.iter().position(|val| *val == C_2).unwrap_or(0);
    let mut idx = 0;
    for (i, val) in dealt_cards.iter().enumerate() {
        if *val == C_2 {
            idx = i;
            break;
        }
    }
    let mut winner = (idx as i32) / (NUM_KC as i32);

    // initializing the flag of "breaking heart"".
    let mut bh_flag = false;

    // When each of the four players has played a card, it is called a "trick";
    // each player plays a card once in a trick.
    for trick in 0..NUM_KC {
        let agent_order = determine_agent_order(winner);

        let mut card_sequence: [i32; NUM_PLAYERS] = [-1; NUM_PLAYERS];

        for turn in 0..NUM_PLAYERS {
            let playing_agent = agent_order[turn] as usize;

            // Letting the agent choose a card.
            let mut card;
            loop {
                card = agents[playing_agent].select_card();
                if is_valid_card(&agents[playing_agent].get_hand(), &card_sequence, card, trick, bh_flag) {
                    break;
                }
            }
            agents[playing_agent].update_hand(card);

            card_sequence[turn] = card;

            let idx = trick * NUM_PLAYERS + turn;
            whole_card_sequence[idx] = card;
            whole_agent_sequence[idx] = playing_agent as i32;

            // When a heart is played for the first time in a game, setting the flag to true.
            if !bh_flag && get_suit(card) == HEART {
                bh_flag = true;
            }
        }

        // The winner of the current trick becomes the leading player of the next trick.
        winner = determine_winner(&agent_order, &card_sequence);
    }

    // A single game ends when NUM_KC tricks have been carried out.
}

fn deal_cards(agents: &mut Vec<Box<dyn Agent>>) -> Vec<i32> {
    let mut v: Vec<i32> = (0..NUM_CARDS as i32).collect();
    loop {
        let mut rng = rand::thread_rng();
        v.shuffle(&mut rng);

        // Prohibiting hearts from appearing 13 times in a row.
        let mut count = 0;
        for i in 0..NUM_CARDS {
            if get_suit(v[i]) == HEART {
                count += 1;
            } else {
                count = 0;
            }
        }
        if count < NUM_KC {
            break;
        }
    }

    for i in 0..NUM_PLAYERS {
        let cards = &v[(i * NUM_KC)..((i + 1) * NUM_KC)];
        agents[i].set_hand(&cards);
    }

    return v;
}

fn determine_agent_order(winner: i32) -> [i32; NUM_PLAYERS] {
    let mut order: [i32; NUM_PLAYERS] = [-1; NUM_PLAYERS];

    for i in 0..NUM_PLAYERS {
        if winner + (i as i32) < (NUM_PLAYERS as i32) {
            order[i] = winner + (i as i32);
        } else {
            order[i] = winner + (i as i32) - (NUM_PLAYERS as i32);
        }
    }

    return order;
}

fn get_suit(card: i32) -> i32 {
    return card / (NUM_KC as i32);
}

fn is_valid_card(
    hand: &[i32; NUM_KC],
    card_sequence: &[i32; NUM_PLAYERS],
    card: i32,
    trick: usize,
    bh_flag: bool,
) -> bool {
    // The first card played in a trick is called the "leading card" and
    // the agent who plays this card is called the "leading player".
    let leading_card = card_sequence[0];

    if leading_card == -1 {
        // In the first trick, only Club-2 can be the leading card.
        if trick == 0 && card != C_2 {
            return false;
        }

        // In the first trick, each agent cannot play a heart.
        if trick == 0 && get_suit(card) == HEART {
            return false;
        }

        // If the leading player has only hearts, it is an exceptional case and the agent may lead with a heart.
        if get_suit(card) == HEART
            && !is_suit_in_hand(hand, CLUB)
            && !is_suit_in_hand(hand, DIA)
            && !is_suit_in_hand(hand, SPADE)
        {
            return true;
        }

        // Until breaking heart occurs, the leading player may not play a heart.
        if !bh_flag && get_suit(card) == HEART {
            return false;
        }

        return true;
    } else {
        // If an agent does not have a card of the same suit as the leading card, the agent play any card.
        if !is_suit_in_hand(hand, get_suit(leading_card)) {
            return true;
        }

        // Each agent must play a card of the same suit as the leading card.
        if get_suit(leading_card) == get_suit(card) {
            return true;
        }

        return false;
    }
}

fn determine_winner(agent_order: &[i32; NUM_PLAYERS], card_sequence: &[i32; NUM_PLAYERS]) -> i32 {
    let mut leading_card = card_sequence[0];
    let lc_suit = get_suit(leading_card);
    let mut winner = agent_order[0];

    // After a trick, the agent who has played the strongest card of the same suit as the leading card
    // is the winner of that trick.

    for (card, agent) in card_sequence.iter().zip(agent_order.iter()) {
        if lc_suit == get_suit(*card) && leading_card < *card {
            leading_card = *card;
            winner = *agent;
        }
    }
    return winner;
}

fn is_suit_in_hand(hand: &[i32; NUM_KC], suit: i32) -> bool {
    for h in hand {
        if *h != -1 && suit == get_suit(*h) {
            return true;
        }
    }
    return false;
}

fn calc_penalty_points(
    card_sequence: &[i32; NUM_CARDS],
    agent_sequence: &[i32; NUM_CARDS],
) -> [i32; NUM_PLAYERS] {
    let mut penalty_points: [i32; NUM_PLAYERS] = [0; NUM_PLAYERS];
    let mut card_subsequence: [i32; NUM_PLAYERS] = [-1; NUM_PLAYERS];
    let mut agent_subsequence: [i32; NUM_PLAYERS] = [-1; NUM_PLAYERS];

    for trick in 0..NUM_KC {
        for turn in 0..NUM_PLAYERS {
            let idx = trick * NUM_PLAYERS + turn;
            card_subsequence[turn] = card_sequence[idx];
            agent_subsequence[turn] = agent_sequence[idx];
        }

        // Each heart equals a one-point penalty and the S-Q equals a 13-point penalty,
        // so the total number of penalty points is 26.
        // The winner of a trick receives all of the penalty points of the cards played in the trick.

        let winner = determine_winner(&agent_subsequence, &card_subsequence) as usize;

        for card in card_subsequence {
            if card >= HEART * (NUM_KC as i32) {
                penalty_points[winner] += 1;
            } else if card == S_Q {
                penalty_points[winner] += 13;
            } else {
            }
        }
    }

    return penalty_points;
}

trait Agent {
    fn get_hand(&self) -> &[i32; NUM_KC];
    fn set_hand(&mut self, cards: &[i32]);
    fn select_card(&mut self) -> i32;
    fn update_hand(&mut self, card: i32);
}

struct RandomAgent {
    hand: [i32; NUM_KC],
}

impl RandomAgent {
    pub fn new() -> Self {
        Self {
            hand: [-1; NUM_KC]
        }
    }

    // pub fn first(&self) -> i32 {
    //     self.hand[0]
    // }
}

impl Agent for RandomAgent {
    fn get_hand(&self) -> &[i32; NUM_KC] {
        &self.hand
    }

    fn get_hand(&self) -> &[i32; NUM_KC] {
        &self.hand
    }

    fn set_hand(&mut self, cards: &[i32]) {
        self.hand = cards.try_into().unwrap();
    }

    // Randomly selecting a card from the hand.
    fn select_card(&mut self) -> i32 {
        let mut rng = rand::thread_rng();
        loop {
            let card_index = rng.gen_range(0..NUM_KC);
            if self.hand[card_index] != -1 {
                return self.hand[card_index];
            }
        }
    }

    fn update_hand(&mut self, card: i32) {
        for i in 0..NUM_KC {
            if self.hand[i] == card {
                self.hand[i] = -1;
                break;
            }
        }
    }
}

struct RuleBasedAgent {
    hand: [i32; NUM_KC],
}

impl RuleBasedAgent {
    pub fn new() -> Self {
        Self {
            hand: [-1; NUM_KC]
        }
    }
}

impl Agent for RuleBasedAgent {
    fn get_hand(&self) -> &[i32; NUM_KC] {
        &self.hand
    }

    fn set_hand(&mut self, cards: &[i32]) {
        self.hand = cards.try_into().unwrap();
    }

    fn select_card(&mut self) -> i32 {
        return 0; // Not yet implemented.
    }

    fn update_hand(&mut self, card: i32) {
        for i in 0..NUM_KC {
            if self.hand[i] == card {
                self.hand[i] = -1;
                break;
            }
        }
    }
}
