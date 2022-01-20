use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::Result;
// use cached::proc_macro::cached;
use clap::Parser;
use rust_decimal::Decimal;
use serde::Deserialize;

macro_rules! zero {
    () => {
        Decimal::from_str("0").unwrap()
    };
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 500)]
    balance: usize,
    #[clap(short, long, default_value_t = 0)]
    dataset: usize,
    #[clap(short, long, default_value_t = 0)]
    algorithme: usize,
}

#[derive(Debug, Deserialize, Clone)]
struct RowBrut {
    name: String,
    price: Decimal,
    profit: Decimal,
}

#[derive(Debug, Deserialize, Clone)]
struct Row {
    name: String,
    price: Decimal,
    profit: Decimal,
    benefits: Decimal,
}

impl Into<Row> for RowBrut {
    fn into(self) -> Row {
        Row {
            name: self.name,
            price: self.price,
            profit: self.profit,
            benefits: self.price * self.profit / Decimal::from_str("100").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
struct Best {
    earnings: Decimal,
    actions: Vec<usize>,
    balance: Decimal,
}

#[derive(Debug, Clone)]
struct RecursiveCached {
    argument: Vec<usize>,
    result: Best,
}

fn get_csv_dataset(file_number: usize) -> Result<Vec<Row>> {
    let file = File::open(format!("dataset/dataset{}_Python+P7.csv", file_number))?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut data: Vec<Row> = Vec::new();
    for result in csv_reader.deserialize() {
        let row: RowBrut = result?;
        data.push(row.into());
    }
    Ok(data)
}

fn optimized_one_loop(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    fn recursive(balance: Decimal, data: &Vec<Row>) -> Best {
        // Considering that the data are sorted by profit (from the best pourcentage
        // to the lowest), then if the actions is not out of budget, then we buy it !
        let mut best = Best {
            earnings: zero!(),
            actions: Vec::new(),
            balance: balance.clone(),
        };
        for (i, row) in data.iter().enumerate() {
            if best.balance >= row.price && !best.actions.contains(&i) {
                best.earnings += row.benefits;
                best.balance -= row.price;
                best.actions.push(i);
            }
        }
        best
    }
    Ok(recursive(balance, &data))
}

fn optimized_recursive(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    let mut cached_recursives: Vec<RecursiveCached> = Vec::new();
    #[derive(Debug, Clone)]
    struct NotCachedError;

    fn get_cache(
        actions: &Vec<usize>,
        cached_recursives: &mut Vec<RecursiveCached>,
    ) -> Result<Best> {
        let mut actions_cloned = actions.clone();
        actions_cloned.sort();
        for data in cached_recursives {
            if data.argument == actions_cloned {
                return Ok(data.result.clone());
            }
        }
        Err(anyhow::anyhow!("Not cached"))
    }
    fn add_cache(actions: &Vec<usize>, best: &Best, cached_recursives: &mut Vec<RecursiveCached>) {
        cached_recursives.push(RecursiveCached {
            argument: actions.clone(),
            result: best.clone(),
        });
    }
    fn recursive(
        stack: usize,
        balance: Decimal,
        earnings: Decimal,
        actions: &Vec<usize>,
        data: &Vec<Row>,
        cached_recursives: &mut Vec<RecursiveCached>,
    ) -> Best {
        // Considering that the data are sorted by profit (from the best pourcentage
        // to the lowest), then if the actions is not out of budget, then we buy it !
        match get_cache(actions, cached_recursives) {
            Ok(result) => {
                println!("was cached");
                return result;
            }
            Err(_) => {}
        }
        let mut current_best = Best {
            earnings,
            actions: actions.clone(),
            balance,
        };
        let mut earnings_increased = zero!();
        // println!(
        //     "Rec balance {} ; actions {:?} ; best {:?} ; earnings {}",
        //     balance, actions, best, earnings
        // );
        // println!("\nRecursive #{}", stack);
        for (i, row) in data.iter().enumerate() {
            if balance < row.price {
                // println!("Not enough capital. #{} => {}", stack, i + 1);
                continue;
            } else if earnings_increased > zero!() {
                // println!("Not enough benefits ! #{} => {}", stack, i + 1);
                break;
            } else if actions.contains(&i) {
                // println!("Action already bought ! #{} => {}", stack, i + 1);
                continue;
            } else {
                let new_balance = balance - row.price;
                let new_earnings = earnings + row.benefits;
                let mut new_actions: Vec<usize> = actions.clone();
                new_actions.push(i.clone());
                if new_earnings > current_best.earnings {
                    // earnings_increased += current_best.earnings - new_earnings;
                    // earnings_increased += new_earnings - current_best.earnings;
                    current_best.earnings = new_earnings.clone();
                    current_best.actions = new_actions.clone();
                    current_best.balance = new_balance.clone();
                    // println!("Improved !")
                }
                // println!(
                //     "Loop #{} : {}, {:?} ; actions_recur {:?} ; incr {}",
                //     stack,
                //     i + 1,
                //     current_best,
                //     new_actions,
                //     earnings_increased,
                // );
                // new_actions.sort();
                let result_best = recursive(
                    stack + 1,
                    new_balance,
                    new_earnings,
                    &new_actions,
                    &data,
                    cached_recursives,
                );
                // let increased_benefits = result_best.earnings - current_best.earnings;
                let increased_benefits = result_best.earnings - current_best.earnings;
                if increased_benefits > zero!() {
                    earnings_increased = increased_benefits;
                    current_best = result_best;
                    // println!(
                    //     "After call Recursive #{} : {}, {:?}",
                    //     stack,
                    //     i + 1,
                    //     current_best
                    // );
                }
                // println!("End loop incr {}", earnings_increased);
            }
        }
        add_cache(actions, &current_best, cached_recursives);
        // println!("Return #{} : {:?}\n", stack, current_best);
        current_best
    }
    Ok(recursive(
        1,
        balance,
        zero!(),
        &Vec::new(),
        &data,
        &mut cached_recursives,
    ))
}

fn brut_force_recursive_binary(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    let mut best = Best {
        earnings: zero!(),
        actions: Vec::new(),
        balance: zero!(),
    };
    let earnings: &mut Decimal = &mut zero!();
    let actions: &mut Vec<usize> = &mut Vec::new();
    let balance: &mut Decimal = &mut balance.clone();

    fn buy_action_and_check_best(
        index: usize,
        data: &Vec<Row>,
        balance: &mut Decimal,
        earnings: &mut Decimal,
        actions: &mut Vec<usize>,
        best: &mut Best,
    ) {
        let row: Row = data[index].clone();
        *balance = *balance - row.price;
        *earnings = *earnings + row.benefits;
        actions.push(index);
        if *earnings > best.earnings && *balance >= zero!() {
            best.earnings = *earnings;
            best.actions = actions.clone().to_owned();
            best.balance = *balance;
        }
    }
    fn remove_action(
        row_index: usize,
        data: &Vec<Row>,
        balance: &mut Decimal,
        earnings: &mut Decimal,
        actions: &mut Vec<usize>,
    ) {
        let row: Row = data[row_index].clone();
        *balance = *balance + row.price;
        *earnings = *earnings - row.benefits;
        let removed_index = actions.pop().expect("Empty vector actions !");
        if removed_index != row_index {
            panic!("Wrong index removed !")
        }
    }

    fn recursive(
        index: usize,
        data: &Vec<Row>,
        balance: &mut Decimal,
        earnings: &mut Decimal,
        actions: &mut Vec<usize>,
        best: &mut Best,
    ) {
        if index < data.len() {
            let row: Row = data[index].clone();
            // println!("index {} {:?}", index, actions);
            // Whithout the action corresponding to the index
            recursive(index + 1, data, balance, earnings, actions, best);

            if *balance > row.price {
                // Whith the action corresponding to the index
                buy_action_and_check_best(index, &data, balance, earnings, actions, best);
                // println!("index {} {:?}", index, actions);
                recursive(index + 1, data, balance, earnings, actions, best);

                // clean before return
                remove_action(index, &data, balance, earnings, actions);
            }
        }
    }

    recursive(0, &data, balance, earnings, actions, &mut best);
    Ok(best)
}

fn brut_force_recursive_redondant(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    let mut best = Best {
        earnings: zero!(),
        actions: Vec::new(),
        balance: zero!(),
    };

    fn recursive(
        balance: Decimal,
        earnings: Decimal,
        actions: &Vec<usize>,
        best: &mut Best,
        // best: &mut Arc<Best>,
        data: &Vec<Row>,
    ) {
        // println!(
        //     "Rec balance {} ; actions {:?} ; best {:?} ; earnings {}",
        //     balance, actions, best, earnings
        // );
        for (i, row) in data.iter().enumerate() {
            if actions.contains(&i) {
                continue;
            } else if balance >= row.price {
                let new_balance = balance - row.price;
                let new_earnings = earnings + row.benefits;
                let mut new_actions = actions.clone();
                new_actions.push(i.clone());
                if new_earnings > best.earnings {
                    best.earnings = new_earnings;
                    best.actions = new_actions.clone().to_owned();
                    best.balance = new_balance;
                }

                recursive(new_balance, new_earnings, &new_actions, best, &data);
            }
        }
    }
    recursive(balance, zero!(), &Vec::new(), &mut best, &data);
    Ok(best)
}

fn check_data(best: &Best, data: Vec<Row>, balance: Decimal) {
    let mut total_earnings = zero!();
    let mut balance = balance;
    for index_row in best.actions.clone() {
        let row = data[index_row].clone();
        total_earnings += row.benefits;
        balance -= row.price;
    }
    assert_eq!(best.earnings, total_earnings);
    assert_eq!(best.balance, balance);
    println!("Checked benefits : {}", total_earnings);
    println!("Checked balance : {}", balance);
}

fn show_result(data: Vec<Row>, best: &Best, duration: Duration) {
    println!("Actions to buy :");
    println!(
        "Brute force, result : {:?} ; duration : {:?}",
        best, duration
    );
    for index_row in &best.actions {
        println!("{}", data[*index_row].name);
    }
}

fn main() -> Result<()> {
    // Parse arguments
    let args: Args = Args::parse();
    let balance: Decimal = args.balance.into();
    let dataset_number = args.dataset;
    let algorithme = args.algorithme;

    let mut data: Vec<Row> = get_csv_dataset(dataset_number)?;

    // sort data by profit (in pourcentage of the price)
    data.sort_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap());
    data.reverse();

    // clean data : removes negative prices and profit
    let data: Vec<Row> = data
        .into_iter()
        .filter(|row| row.price > zero!() && row.profit > zero!())
        .collect();

    // Start benchmark's clock
    let start = Instant::now();

    // Run the algorithme choosen in arguments
    let best = match algorithme {
        0 => brut_force_recursive_binary(data.clone(), balance)?,
        1 => brut_force_recursive_redondant(data.clone(), balance)?,
        2 => optimized_recursive(data.clone(), balance)?,
        3 => optimized_one_loop(data.clone(), balance)?,
        val => panic!("algorithme number {} does not exist.", val),
    };

    // Result benchmark's clock
    let end = Instant::now();
    let duration = end.duration_since(start);
    show_result(data.clone(), &best, duration);

    // Verify the coherence of the result
    check_data(&best, data.clone(), balance);

    Ok(())
}
