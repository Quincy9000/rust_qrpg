use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sqlite::{open, Connection, State};
use std::collections::HashMap;
use std::io::{Read, Write};

/** Test Comment
 * ! Alert
 * ? Question
 * * Important
 * TODO: Do stuff
 */
pub fn get_db() -> Connection {
    open("qrpg.db").unwrap()
}

const PLAYERS: &str = "Players/";

pub fn dir_exists() {
    if let Err(_) = std::fs::read_dir(PLAYERS) { 
        std::fs::create_dir(PLAYERS).expect("file dir err");
    }
}

pub fn enemies_from_db(db: &Connection) -> Vec<Enemy> {
    let mut v = Vec::new();
    let mut statement = db.prepare("SELECT * FROM enemies").unwrap();
    loop  {
        match statement.next() {
            Ok(State::Row) => {
                    v.push(Enemy::new(&statement.read::<String>(0).unwrap()));
                }
            _ => break v,
        }
    }
}

pub fn weapons_from_db(db: &Connection) -> Vec<Weapon> {
    let mut v = Vec::new();
    let mut statement = db.prepare("SELECT * FROM weapons").unwrap();
    loop {
        match statement.next() {
            Ok(State::Row) => {
                let name: String = statement.read(0).unwrap();
                let weight: i64 = statement.read(1).unwrap();
                let value: i64 = statement.read(2).unwrap();
                let p: f64 = statement.read(3).unwrap();
                let t: f64 = statement.read(4).unwrap();
                let m: f64 = statement.read(5).unwrap();
                let w = Weapon {
                    item: Item::new(&name, weight as f32, value as i32),
                    physique_scale: p as f32,
                    technique_scale: t as f32,
                    mystique_scale: m as f32,
                };
                v.push(w);
            },
            _ => break v,
        }
    }
}

fn clear() {
    console::Term::stdout().clear_screen().unwrap();
}

fn any_key(msg: &str) {
    println!("{}", msg);
    console::Term::stdout().read_key().unwrap();
}

fn input(msg: &str) -> String {
    println!("{}", msg);
    let mut s = String::new();
    match std::io::stdin().read_line(&mut s) {
        Ok(_) => s.trim().into(),
        Err(_) => s,
    }
}

fn choice<T: std::fmt::Display, F: Fn()>(display: F, options: &[T], quit: bool) -> i32 {
    let mut selection: i32 = 0;
    loop {
        clear();
        display();
        if quit {
            println!("Type Q to quit.");
        }
        for (n, v) in options.iter().enumerate() {
            if selection == n as i32 {
                println!("{}: {} <-", n + 1, v);
            } else {
                println!("{}: {}", n + 1, v);
            }
        }
        let k = console::Term::stdout()
            .read_key()
            .expect("Failed to key: in choice()");
        if k == console::Key::ArrowUp {
            selection -= 1;
        }
        if k == console::Key::ArrowDown {
            selection += 1;
        }
        if quit && k == console::Key::Char('q') {
            selection = -1;
            break selection;
        }
        for c in "1234567890".chars() {
            if k == console::Key::Char(c) {
                selection = c.to_string().parse::<i32>().unwrap() - 1;
            }
        }
        if k == console::Key::Enter {
            break selection;
        }
        if selection < 0 {
            selection = 0;
        }
        if selection > options.len() as i32 - 1 {
            selection = options.len() as i32 - 1;
        }
    }
}

pub trait Combatant {
    fn get_stats(&self) -> Stats;
    fn name(&self) -> &str;
}

pub trait Attacker: Combatant {
    fn damage(&self) -> i32 {
        0
    }
}

pub trait Defender: Combatant {
    fn defense(&self) -> i32 {
        0
    }
    fn take_damage(&mut self, damage: i32);
}

#[derive(Default, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub physique: i32,
    pub technique: i32,
    pub mystique: i32,
}

impl Stats {
    pub fn new<T: Into<i32>>(physique: T, technique: T, mystique: T) -> Self {
        let physique = physique.into();
        let mystique = mystique.into();
        let technique = technique.into();
        Self {
            physique,
            technique,
            mystique,
        }
    }

    pub fn max_health(&self) -> i32 {
        (self.physique * 5) + (self.technique * 3) + (self.mystique * 4) + 20
    }

    pub fn max_stamina(&self) -> i32 {
        (self.physique * 4) + (self.technique * 5) + (self.mystique * 3) + 20
    }

    pub fn max_mana(&self) -> i32 {
        (self.physique * 3) + (self.technique * 4) + (self.mystique * 5) + 20
    }

    pub fn carry_capacity(&self) -> i32 {
        10 + (self.physique as i32 * 5)
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Physique: {}, Technique: {}, Mystique: {}",
            self.physique, self.technique, self.mystique
        ))
    }
}

#[derive(Default, Debug, PartialOrd, PartialEq, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub weight: f32,
    pub value: i32,
}

impl Item {
    pub fn new(name: &str, weight: f32, value: i32) -> Self {
        let name = name.to_owned();
        Self {
            name,
            weight,
            value,
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.name))
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Clone)]
pub struct Weapon {
    pub item: Item,
    pub physique_scale: f32,
    pub technique_scale: f32,
    pub mystique_scale: f32,
}

impl Weapon {
    pub fn name(&self) -> &str {
        &self.item.name
    }

    pub fn damage(&self, attacker: &dyn Attacker) -> i32 {
        ((self.physique_scale * attacker.get_stats().physique as f32)
            + (self.technique_scale * attacker.get_stats().technique as f32)
            + (self.mystique_scale * attacker.get_stats().mystique as f32)) as i32
    }
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&format!(
            "{}, Physique: {}, Technique: {}, Mystique: {}, Value: {}, Weight: {}",
            self.item,
            self.physique_scale,
            self.technique_scale,
            self.mystique_scale,
            self.item.value,
            self.item.weight,
        ))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum Contents {
    Item(Item),
    Weapon(Weapon),
}

impl std::fmt::Display for Contents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let fmt = match self {
            Contents::Weapon(w) => format!("{}", w),
            Contents::Item(i) => format!("{}", i),
        };
        f.write_str(&fmt)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub location: String,
    pub quest: String,
    pub stats: Stats,
    pub health: i32,
    pub stamina: i32,
    pub mana: i32,
    pub money: i32,
    pub inventory: Vec<Contents>,
    pub equipped: Option<Weapon>,
    pub triggers: HashMap<String, bool>,
}

impl Player {
    pub fn new(name: &str, stats: Stats) -> Self {
        Self {
            name: name.into(),
            stats,
            health: stats.max_health() as i32,
            stamina: stats.max_stamina() as i32,
            mana: stats.max_mana() as i32,
            inventory: Vec::new(),
            equipped: Some(Weapon {
                item: Item::new("Hands", 0.0, 0),
                physique_scale: 1.0,
                technique_scale: 1.0,
                mystique_scale: 1.0,
            }),
            location: "None".into(),
            quest: "None".into(),
            money: 100,
            triggers: HashMap::new(),
        }
    }

    pub fn to_file(&self) -> std::io::Result<&str> {
        let path = format!("{}{}{}", PLAYERS, self.name.clone(), ".txt");
        let s = serde_json::to_string(self)?;
        let mut file = std::fs::File::create(&path)?;
        file.write(s.as_bytes())?;
        Ok("Ok!")
    }

    pub fn from_file(path: &str) -> std::io::Result<Player> {
        let mut file = std::fs::File::open(format!("{}{}", PLAYERS, path))?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let p = serde_json::from_str(&s)?;
        Ok(p)
    }

    pub fn create_random() -> Self {
        let name = "Default";
        let mut rng = thread_rng();
        let stats = Stats::new(
            rng.gen_range(1, 6),
            rng.gen_range(1, 6),
            rng.gen_range(1, 6),
        );
        Player::new(&name, stats)
    }

    pub fn get_items(&self) -> String {
        if self.inventory.is_empty() {
            "None".into()
        } else {
            let mut items = String::new();
            for i in self.inventory.iter() {
                match i {
                    Contents::Item(item) => items.push_str(&format!("{}, ", item.name)),
                    Contents::Weapon(wep) => items.push_str(&format!("{}, ", wep.item.name)),
                }
            }
            items.pop();
            items.pop();
            items
        }
    }

    pub fn create_character() -> Self {
        clear();
        let name = input("What is the name of your character?");
        let mut points = 5;
        let mut stats = Stats::new(1, 1, 1);

        while points > 0 {
            clear();
            let mut msg = String::from("Physique: Physical Damage, and major for Combat type people.\nTechnique: Technical Damage, and major for Agile type people.\nMystique: Mystical Damage, and major for Magic type people.\n");

            if points == 5 {
                msg += "Where do you want your first point to go?\n";
            } else if points > 1 {
                msg += "Where do you want the next point to go?\n";
            } else {
                msg += "Where do you want the last point to go?\n";
            }

            msg += &format!(
                "Max Health: {}\nMax Stamina: {}\nMax Mana: {}\nPoints left: {}\nChoose Stats: ",
                stats.max_health(),
                stats.max_stamina(),
                stats.max_mana(),
                points
            );

            let point = choice(
                || println!("{}", msg),
                &["Physique", "Technique", "Mystique"],
                false,
            );

            if point == 0 {
                stats.physique += 1;
                points -= 1;
            } else if point == 1 {
                stats.technique += 1;
                points -= 1;
            } else {
                stats.mystique += 1;
                points -= 1;
            }
        }
        Player::new(&name, stats)
    }
}

impl Combatant for Player {
    fn get_stats(&self) -> Stats {
        self.stats
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Attacker for Player {
    fn damage(&self) -> i32 {
        match &self.equipped {
            Some(w) => w.damage(self),
            None => 1,
        }
    }
}

impl Defender for Player {
    fn defense(&self) -> i32 {
        self.get_stats().physique as i32
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "(name: {}, stats: {}, health: {}, stamina: {}, mana: {}, money: {}, quest: {}, location: {}, triggered: {:?})",
            self.name, self.stats, self.health, self.stamina, self.mana, self.money, self.quest, self.location, self.triggers,
        ))
    }
}

#[derive(Default, Debug)]
pub struct Enemy {
    pub name: String,
    pub stats: Stats,
    pub health: i32,
    pub weapon: Option<Weapon>,
}

impl Enemy {
    pub fn random() -> Self {
        let db = get_db();
        let mut enemies = enemies_from_db(&db);
        let n: usize = rand::thread_rng().gen_range(0, enemies.len());
        enemies.remove(n)
    }

    pub fn with_stats(&self, p: i32, t: i32, m: i32) -> Self {
        let s = Stats {
            physique: p,
            technique: t,
            mystique: m,
        };
        Enemy {
            name: self.name.clone(),
            stats: s,
            health: s.max_health(),
            weapon: None,
        }
    }

    pub fn with_weapon<T: Into<f32>>(
        &self,
        weapon: &str,
        p: T,
        t: T,
        m: T,
        weight: T,
        value: i32,
    ) -> Self {
        Enemy {
            name: self.name.clone(),
            stats: self.stats,
            health: self.health,
            weapon: Some(Weapon {
                physique_scale: p.into(),
                technique_scale: t.into(),
                mystique_scale: m.into(),
                item: Item::new(weapon, weight.into(), value),
            }),
        }
    }

    pub fn new<T: AsRef<str>>(name: T) -> Self {
        Enemy {
            name: name.as_ref().into(),
            stats: Default::default(),
            health: 100,
            weapon: None,
        }
    }
}

impl std::fmt::Display for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&format!(
            "Enemy{{ name: {}, stats:{} }}",
            self.name, self.stats
        ))
    }
}

impl Combatant for Enemy {
    fn get_stats(&self) -> Stats {
        self.stats
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Attacker for Enemy {
    fn damage(&self) -> i32 {
        let stats = self.get_stats();
        stats.physique as i32
    }
}

impl Defender for Enemy {
    fn defense(&self) -> i32 {
        (self.name.len() / 2usize) as i32
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }
}

pub struct BattleOutcome<'a> {
    pub attacker: &'a dyn Attacker,
    pub defender: &'a dyn Defender,
    pub damage: i32,
}

impl<'a> BattleOutcome<'a> {
    pub fn print(&self) {
        println!("{}", self);
    }
}

impl<'a> std::fmt::Display for BattleOutcome<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} attacked {}, for {} damage!",
            self.attacker.name(),
            self.defender.name(),
            self.damage
        ))
    }
}

pub fn combat<'a>(attacker: &'a dyn Attacker, defender: &'a mut dyn Defender) -> BattleOutcome<'a> {
    let attacker_damage = attacker.damage();
    let defender_defense = defender.defense();
    println!("tset");
    let mut damage = attacker_damage - defender_defense;
    if damage > 0 {
        defender.take_damage(damage);
    } else {
        damage = 1;
        defender.take_damage(1);
    }
    BattleOutcome {
        attacker,
        defender,
        damage,
    }
}

pub fn buy(mut player: Player) -> Player {
    let db = get_db();
    let weapons = weapons_from_db(&db);
    'l: loop {
        let f = format!("What ye be wantin to buy?\nHere are thee weapons I have to offer ye'!\nYe have ${}.\nYour inventory [{}].", player.money, player.get_items());
        let c = choice(|| println!("{}", f), &weapons, true);
        for (n, w) in weapons.iter().enumerate() {
            if c == n as i32 {
                let a = choice(
                    || {
                        println!(
                            "{}",
                            &format!("Ye want to buy a {}, for ${}?", w, w.item.value)
                        )
                    },
                    &["yes", "no"],
                    false,
                );

                if a == 0 {
                    if player.money >= w.item.value {
                        println!("That shall serve you well!");
                        player.money -= w.item.value;
                        player.inventory.push(Contents::Weapon(w.clone()));
                    } else {
                        any_key("You idiot! You can't afford that, ye swindler!");
                    }
                }
            } else if c < 0 {
                break 'l;
            }
        }
    }
    player
}

pub fn sell(mut player: Player) -> Player {
    if player.inventory.is_empty() {
        any_key("Ye can't sell, if ye has no valuables!");
        return player;
    }
    loop {
        let f = format!("What're ye sellin'!\nYour money ${}", player.money);
        let c = choice(|| println!("{}", &f), &player.inventory, true);
        let mut remove = -1;
        if c == -1 {
            break player;
        }
        for (n, w) in player.inventory.iter().enumerate() {
            if let Contents::Weapon(wep) = w {
                if c == n as i32 {
                    let f = format!(
                        "I'll take ye, {} for ${}\nYe be sure, ye want to sell thee?",
                        wep.item.name, wep.item.value
                    );
                    let c = choice(|| println!("{}", &f), &["yes", "no"], true);
                    if c == 0 {
                        //weapon index to remove later
                        remove = n as i32;
                        player.money += wep.item.value;
                        break;
                    }
                }
            }
        }
        if remove != -1 {
            player.inventory.remove(remove as usize);
        }
    }
}

pub fn shop(mut player: Player) -> Player {
    loop {
        let c = choice(
            || println!("Welcome to ye ol' shoppe! What must ye be buyin, or sellin?.."),
            &["Buy", "Sell"],
            true,
        );
        if c == 0 {
            player = buy(player);
        } else if c == 1 {
            player = sell(player);
        } else {
            println!("Cya later buddy!");
            break;
        }
    }
    player
}

pub fn select_equipped(mut player: Player) -> Player {
    if player.inventory.is_empty() {
        return player;
    }
    loop {
        let f = format!(
            "Do you want to change your equipped weapon?\nYour current one is: {}\n",
            player.equipped.as_ref().unwrap()
        );
        let c = choice(|| println!("{}", &f), &["yes", "no"], true);
        if c == 0 {
            let c = choice(
                || println!("Switch to which weapon?"),
                &player.inventory,
                true,
            );
            let mut swap = false;
            let mut index = 0;
            for (n, w) in player.inventory.iter().enumerate() {
                if let Contents::Weapon(wep) = w {
                    if c == n as i32 {
                        println!("Swapped to the {}!", &wep.item.name);
                        index = n;
                        swap = true;
                        break;
                    }
                }
            }
            if swap {
                if let Contents::Weapon(wep) = player.inventory.remove(index) {
                    let weapon = Weapon {
                        item: wep.item,
                        physique_scale: 1.0,
                        technique_scale: 1.0,
                        mystique_scale: 1.0,
                    };
                    player
                        .inventory
                        .push(Contents::Weapon(player.equipped.take().unwrap()));
                    player.equipped = Some(weapon);
                }
            }
        } else {
            break;
        }
    }
    player
}

pub fn script_battle(player: &mut Player, enemy: &mut Enemy) {
    'battle: loop {
        let dsp = || {
            println!("This is the battle screen!");
            println!("==========================");
            println!("(\\_/)\n(>.<)\n(\")_(\")\n");
            println!("Bunny is about to strike!(Strikes first)");
            println!("Bunny HP: {}\n", enemy.health);
            println!("HP: {}", player.health);
            println!("SP: {}", player.stamina);
            println!("MP: {}", player.mana);
            println!("==========================");
        };

        if player.health < 1 {
            clear();
            dsp();
            any_key("You died! Game over!");
            break 'battle;
        }

        if enemy.health < 1 {
            clear();
            dsp();
            any_key("You win! Enemy died!");
            break 'battle;
        }

        let c = choice(dsp, &["Attack", "Item", "Flee"], false);

        if c == 0 {
            if enemy.stats.technique >= player.stats.technique {
                combat(enemy, player).print();
                if player.health > 0 {
                    combat(player, enemy).print();
                }
            } else {
                combat(player, enemy).print();
                if enemy.health > 0 {
                    combat(enemy, player).print();
                }
            }
            any_key("");
        } else if c == 1 {
            any_key("You dont have any items because you were mugged..");
        } else {
            any_key(
                "You try to flee, but the bunny overpowers you, and forces you to magically fight!",
            );
        }
    }
    println!("{}", enemy);
}

pub fn char_intro(mut player: Player) -> Player {
    any_key("Ahoy there, traveler! Would ye be interested in helpn' dis ol' merchant with a task?");
    any_key("The task be simple, ya! You help me travel to the next city over yonder. (Points eastwards)");
    any_key("Then i'll pay yee when we get to the city, ya?");
    any_key("Alright! Sounds great. Let's get going'");
    any_key("Hours later after traveling for the rest of the day. You wake up with masked shadow figures over your tent!");
    any_key("They attack you visciously, knock you out, and take all your belongings.");
    any_key("You feel a massive splash of water as you go in and out of conciousness.");
    any_key("You wake up hours later..With no food and water..");
    any_key("Those bastards took all of your equipment, you need to head to the nearest town to fully recover..");
    any_key("As you fumble around along a dirt path back to any nearby civilization..you hear rustling in the bushes from the forst!");
    any_key("You get ready for anythin!");
    any_key("Out of the bushes come a tiny, but a rabid and agitated animal ready to strike!");
    any_key("You must fight it off or die! Even if you only have half of your strength left..");
    player.health = (player.stats.max_health() / 2) as i32;
    let mut bunny = Enemy::new("Rabbit").with_stats(1, 1, 1);
    script_battle(&mut player, &mut bunny);
    player.triggers.insert("char_intro".into(), true);
    player.to_file().expect("error to file char intro");
    player
}

pub fn story(player: Player) -> Player {
    let mut player = player;
    if player.triggers.is_empty() {
        player = char_intro(player);
    }
    player
}

fn menu() {
    loop {
        let f = format!("*_*_*_*_*_*_*_*_*_*_*\nWelcome to Quincy RPG\n*_*_*_*_*_*_*_*_*_*_*\n");
        let c = choice(
            || println!("{}", &f),
            &["New Game", "Load Game", "View Character", "Options", "Exit"],
            false,
        );
        if c == 0 {
            println!("new game");
            let p = Player::create_character();
            p.to_file().unwrap();
            any_key("Character created!");
            story(p);
        } else if c == 1 {
            let player = input("What character do you want to load?");
            let player = match Player::from_file(&(player + ".txt")) {
                Ok(o) => o,
                Err(_) => {
                    any_key("Can't load that character...");
                    continue;
                }
            };
            println!("{}", player);
            any_key("load game");
        } else if c == 2 {
            let player = input("What character do you want to load?") + ".txt";
            let player = match Player::from_file(&player) {
                Ok(p) => p,
                Err(_) => {
                    any_key("Failed to load character.");
                    continue;
                }
            };
            any_key(&format!("{}", player));
        } else if c == 3 {
            println!("options");
        } else {
            break;
        }
    }
}

fn main() {
    dir_exists();
    menu();
}

mod test {
    #[test]
    fn test1() {
        let p = crate::Player::create_random();
        p.to_file().expect("OH NO");
        let p = crate::Player::from_file("Default.txt").expect("OH NO 2");
        println!("{:?}", p);
    }

    #[test]
    fn test2() {
        let e = crate::Enemy::random();
        println!("{}", e);
    }

    #[test]
    fn test3() {
        let p = crate::Player::create_random();
        println!("{:?}", p);
        let p = crate::shop(p);
        println!("{:?}", p);
    }

    #[test]
    fn test4() {
        let db = crate::get_db();
        let weps = crate::weapons_from_db(&db);
        println!("{:?}", weps);
    }

    #[test]
    fn test5() {
        use crate::{Attacker, Enemy};
        let enemy = Enemy::new("Rabbit").with_stats(10, 5, 6);
        let damage: i32 = enemy.damage();
        println!("{}, damage: {}", enemy, damage);
    }
}
