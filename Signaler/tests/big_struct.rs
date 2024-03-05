use decorators::*;
use signal::*;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

use test_log::test;

use std::time::Instant;

use rand::prelude::*;
use rand_derive2::RandGen;
use rnglib::{Language, RNG};

#[derive(Clone, Debug, RandGen, PartialEq)]
enum Race {
    Dragonborn,
    Dwarf,
    Elf,
    Gnome,
    HalfElf,
    Human,
    Patrick,
}

impl Race {
    fn to_random_lang(&self) -> &Language {
        let mut rng = rand::thread_rng();
        match self {
            Self::Dragonborn => [
                Language::Demonic,
                Language::Elven,
                Language::Fantasy,
                Language::Roman,
            ]
            .choose(&mut rng)
            .unwrap(),
            Self::Dwarf => [
                Language::Curse,
                Language::Demonic,
                Language::Fantasy,
                Language::Roman,
            ]
            .choose(&mut rng)
            .unwrap(),
            Self::Elf => &Language::Elven,
            Self::Gnome => [Language::Fantasy, Language::Roman]
                .choose(&mut rng)
                .unwrap(),
            Self::HalfElf => [Language::Elven, Language::Fantasy, Language::Roman]
                .choose(&mut rng)
                .unwrap(),
            Self::Human => [Language::Fantasy, Language::Roman]
                .choose(&mut rng)
                .unwrap(),
            Self::Patrick => unreachable!(),
        }
    }
}

fn new_name(race: &Race) -> String {
    let mut rng = thread_rng();
    if *race == Race::Patrick {
        "Patrick".into()
    } else {
        let rngrpg = RNG::try_from(race.to_random_lang()).unwrap();
        format!(
            "{} {}",
            rngrpg.generate_name_by_count(rng.gen_range(2..5)),
            rngrpg.generate_name_by_count(rng.gen_range(2..5))
        )
    }
}

#[derive(Clone, Debug, RandGen)]
enum Item {
    Longsword,
    Dagger,
    Greataxe,
    Club,
    Chainmail,
    LeatherArmor,
    PlateArmor,
    HealingPotion,
    PotionOfStrength,
    AdventurersGear,
    ScrollOfFireball,
}

#[derive(Clone, Debug, Default, Signaler)]
struct Stats {
    #[property]
    strength: u32,
    #[property]
    dexterity: u32,
    #[property]
    intelligence: u32,
    #[property]
    health: u32,
    #[property]
    mana: u32,
}

impl Stats {
    fn level_up(&mut self) {
        let mut rng = thread_rng();

        self.strength += rng.gen_range(0..3);
        self.dexterity += rng.gen_range(0..3);
        self.intelligence += rng.gen_range(0..3);
        self.health += rng.gen_range(2..6);
        self.mana += rng.gen_range(2..6);
    }
}

#[derive(Clone, Debug, Default, Signaler)]
struct Inventory {
    #[property]
    items: Vec<Item>,
    #[property]
    max_capacity: usize,
}

#[derive(Debug, Signaler)]
struct Character {
    #[property]
    name: String,
    #[property]
    level: u32,
    #[property]
    stats: Stats,
    #[property]
    race: Race,
    inventory: InventorySignaler,
}

impl InventorySignaler {
    fn create(level: u32) -> Self {
        let mut rng = thread_rng();
        Self {
            data: Inventory {
                items: (0..rand::thread_rng().gen_range(0..8))
                    .map(|_| rand::random())
                    .collect(),
                max_capacity: rng.gen_range(9..20 + 2 * level) as usize,
            },
            ..Default::default()
        }
    }
}

impl std::fmt::Debug for InventorySignaler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inventory")
            .field("items", &self.data.items)
            .field("max_capacity", &self.data.max_capacity)
            .finish()
    }
}

impl Default for Character {
    fn default() -> Self {
        let mut rng = thread_rng();
        let level = rng.gen_range(1..13);

        let race: Race = rand::random();
        let name = new_name(&race);
        Self {
            name,
            level,
            race,
            stats: Stats {
                strength: rng.gen_range(4..18 + level),
                dexterity: rng.gen_range(4..18 + level),
                intelligence: rng.gen_range(4..18 + level),
                health: rng.gen_range(7..20 + 3 * level),
                mana: rng.gen_range(7..20 + 3 * level),
            },
            inventory: InventorySignaler::create(level),
        }
    }
}

impl CharacterSignaler {
    fn level_up(&mut self) {
        self.data.level += 1;
        self.data.stats.level_up();
        self.data.inventory.data.max_capacity += 1;
        self.emit_level();
        self.emit_stats();
        self.data.inventory.emit_max_capacity();
    }
}

#[test]
fn test_joao_hypothesis_rpg() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        const SIZE: usize = 10;
        let mut tasks = [(); SIZE].map(|_| CharacterSignaler::default());
        let start = Instant::now();
        for task in &mut tasks {
            task.level_up();
        }
        println!(
            "Time elapsed in rpg {} series emission: {:?}",
            SIZE,
            start.elapsed()
        );
        sleep(Duration::from_millis(100)).await;
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}
