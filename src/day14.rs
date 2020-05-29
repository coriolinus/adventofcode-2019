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
        let reactions = parse::<Reaction>(path).unwrap().collect::<Vec<_>>();
        let producers: HashMap<_, _> = reactions.iter().map(|reaction| (reaction.outputs.elem.clone(), reaction.clone())).collect();
        assert!(producers.contains_key(FUEL), "fuel must be an output");

        let mut want = VecDeque::new();
        want.push_back(Reagent{qty: 1, elem: FUEL.into()});
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

        println!("ore per 1 fuel: {}", ore_used);
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Reagent {
    qty: u32,
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
            qty: tokens[0].parse::<u32>().map_err(|e| e.to_string())?,
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
    used: u32,
    total_outputs_wanted: u32,
}

impl ReactionQty {
    fn add_wanted_outputs(&mut self, additional_outputs: u32, outputs_per_reaction: u32) -> u32 {
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