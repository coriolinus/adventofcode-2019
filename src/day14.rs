use crate::{
    parse, CommaSep, Exercise,
};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::str::FromStr;

const ORE: &'static str = "ORE";
const FUEL: &'static str = "FUEL";

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let producers = make_producers(path);
        let ore_used = ore_for(1, &producers);

        println!("ore per 1 fuel: {}", ore_used);
    }

    fn part2(&self, path: &Path) {
        const ORE_MINED: u64 = 1000000000000;
        let producers = make_producers(path);

        // binary search guess and check to find the greatest amount of fuel
        // refinable with this much ore
        let mut low = ORE_MINED / ore_for(1, &producers);
        let mut high = low*2;
        let mut prev_guess = 0;
        let mut guess = (low + high) / 2;

        while guess != prev_guess {
            #[cfg(feature = "debug")]
            dbg!(low, guess, high);

            let needed = ore_for(guess, &producers);
            match needed.cmp(&ORE_MINED) {
                std::cmp::Ordering::Equal => break,
                std::cmp::Ordering::Less => low = guess,
                std::cmp::Ordering::Greater => high = guess,
            }
            prev_guess = guess;
            guess = (low + high) / 2;
        }

        println!("max fuel for 1 trillion ore: {}", guess);
    }
}

fn make_producers(path: &Path) -> HashMap<String, Reaction> {
    let reactions = parse::<Reaction>(path).unwrap().collect::<Vec<_>>();
    let producers: HashMap<_, _> = reactions.into_iter().map(|reaction| (reaction.outputs.elem.clone(), reaction)).collect();
    assert!(producers.contains_key(FUEL), "fuel must be an output");
    producers
}

fn ore_for(fuel: u64, producers: &HashMap<String, Reaction>) -> u64 {
    let mut want = VecDeque::new();
    want.push_back(Reagent{qty: fuel, elem: FUEL.into()});
    let mut reactions_used: HashMap<String, ReactionQty> = HashMap::new();


    while let Some(reagent) = want.pop_front() {
        let reaction = producers.get(&reagent.elem).expect("reaction wasn't found!");
        let increased_reactions = reactions_used.entry(reagent.elem).or_default().add_wanted_outputs(reagent.qty, reaction.outputs.qty);
        if increased_reactions > 0 {
            for mut input in reaction.inputs.iter().cloned() {
                if input.elem != ORE {
                    input.qty *= increased_reactions;
                    want.push_back(input);
                }
            }
        }
    }

    let mut ore_used = 0;
    for (product, qty) in reactions_used {
        let reaction = producers.get(&product).unwrap();
        for input in reaction.inputs.iter() {
            if input.elem == ORE {
                ore_used += qty.used * input.qty;
            }
        }
    }

    ore_used
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Reagent {
    qty: u64,
    elem: String,
}

impl FromStr for Reagent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s.trim().split_whitespace().collect::<Vec<_>>();
        if tokens.len() != 2 {
            Err(format!("reagent expects 2 tokens; got {}", tokens.len()))?;
        }
        Ok(Reagent {
            qty: tokens[0].parse::<u64>().map_err(|e| e.to_string())?,
            elem: tokens[1].into(),
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Reaction {
    inputs: Vec<Reagent>,
    outputs: Reagent,
}

impl FromStr for Reaction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split("=>").collect::<Vec<_>>();
        if parts.len() != 2 {
            Err(format!("reaction expects 2 parts; got {}", parts.len()))?;
        }
        let reaction = Reaction{
            inputs: parts[0].parse::<CommaSep<Reagent>>()?.0,
            outputs: parts[1].parse()?,
        };
        if reaction.inputs.is_empty() {
            Err("reaction has no inputs".to_string())?;
        }
        Ok(reaction)
    }
}

#[derive(Default, Debug)]
struct ReactionQty {
    used: u64,
    total_outputs_wanted: u64,
}

impl ReactionQty {
    fn add_wanted_outputs(&mut self, additional_outputs: u64, outputs_per_reaction: u64) -> u64 {
        let old_used = self.used;
        self.total_outputs_wanted += additional_outputs;
        if self.used * outputs_per_reaction < self.total_outputs_wanted{
            self.used = self.total_outputs_wanted / outputs_per_reaction;
            // we need to increment this if there's not an even division
            if self.used * outputs_per_reaction < self.total_outputs_wanted {
                self.used += 1;
            }
        }

        self.used - old_used
    }
}