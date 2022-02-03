use std::cmp::{max, min};
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::Result;
use plotters::prelude::*;
// use plotters::prelude::{
//     AreaSeries, BitMapBackend, ChartBuilder, Color, IntoDrawingArea, LabelAreaPosition,
//     PathElement, SeriesLabelPosition, BLACK, BLUE, RED, WHITE,
// };
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
    #[clap(short, long, takes_value = false)]
    curves: bool,
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

#[allow(unused)]
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

fn optimized_recursive_stack(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    fn recursive(stack: usize, balance: Decimal, earnings: Decimal, data: &Vec<Row>) -> Best {
        // Considering that the data are sorted by profit (from the best pourcentage
        // to the lowest), then if the actions is not out of budget, then we buy it !
        let mut current_best = Best {
            earnings: zero!(),
            actions: Vec::new(),
            balance: zero!(),
        };
        let mut earnings_increased = zero!();
        // println!(
        //     "Rec balance {} ; actions {:?} ; best {:?} ; earnings {}",
        //     balance, actions, best, earnings
        // );
        // println!("\nRecursive(#{}, {}€, {}€)", stack, balance, earnings);
        for i in stack..data.len() {
            let row = data[i].clone();
            if balance < row.price {
                // println!(
                //     "Not enough capital. {}€<{}€ ; #{}:{}",
                //     balance, row.price, stack, i
                // );
                continue;
            } else if earnings_increased > zero!() {
                // println!("Not enough benefits ! #{}:{}", stack, i);
                break;
            } else {
                // println!("Loop #{}:{} ; {:?}", stack, i, row);
                let mut actions: Vec<usize> = Vec::new();
                let new_balance = balance - row.price;
                let new_earnings = earnings + row.benefits;
                // if new_earnings > current_best.earnings {
                current_best.earnings = new_earnings.clone();
                current_best.actions.push(i);
                actions.push(i);
                current_best.balance = new_balance.clone();
                // println!("Inproved loop after ! {:?}", current_best);
                // }
                // println!(
                //     "Start recursive #{}:{} => {:?} ; actions {:?}",
                //     stack, i, current_best, actions
                // );
                let result_best = recursive(i + 1, new_balance, new_earnings, &data);
                let increased_benefits = result_best.earnings - current_best.earnings;
                if increased_benefits > zero!() {
                    // println!("Inproved Réc ! {:?} => {:?}", current_best, result_best);
                    earnings_increased = increased_benefits;
                    actions.append(&mut result_best.actions.clone());
                    current_best = result_best;
                    current_best.actions = actions;
                    // println!("Inproved Réc after ! {:?}", current_best);
                }
                // println!("End loop incr {}", earnings_increased);
            }
        }
        // println!("Return #{} : {:?}\n", stack, current_best);
        current_best
    }
    Ok(recursive(0, balance, zero!(), &data))
}

fn optimized_recursive(data: Vec<Row>, balance: Decimal) -> Result<Best> {
    let mut cached_recursives: Vec<RecursiveCached> = Vec::new();
    #[derive(Debug, Clone)]
    struct NotCachedError;

    //     fn get_cache(
    //         actions: &Vec<usize>,
    //         cached_recursives: &mut Vec<RecursiveCached>,
    //     ) -> Result<Best> {
    //         let mut actions_cloned = actions.clone();
    //         actions_cloned.sort();
    //         for data in cached_recursives {
    //             if data.argument == actions_cloned {
    //                 return Ok(data.result.clone());
    //             }
    //         }
    //         Err(anyhow::anyhow!("Not cached"))
    //     }
    //     fn add_cache(actions: &Vec<usize>, best: &Best, cached_recursives: &mut Vec<RecursiveCached>) {
    //         cached_recursives.push(RecursiveCached {
    //             argument: actions.clone(),
    //             result: best.clone(),
    //         });
    //     }
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
        // match get_cache(actions, cached_recursives) {
        //     Ok(result) => {
        //         println!("was cached");
        //         return result;
        //     }
        //     Err(_) => {}
        // }
        let mut current_best = Best {
            earnings,
            actions: actions.clone(),
            balance,
        };
        let mut earnings_increased = zero!();
        // println!("\nRecursive #{}", stack);
        for (i, row) in data.iter().enumerate() {
            if actions.contains(&i) {
                // println!("Not enough capital. #{} => {}", stack, i + 1);
                continue;
            } else if balance < row.price {
                // println!("Action already bought ! #{} => {}", stack, i + 1);
                continue;
            } else if earnings_increased > zero!() {
                // println!("Not enough benefits ! #{} => {}", stack, i + 1);
                break;
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
                }
                // println!(
                //     "Loop #{} : {}, {:?} ; actions_recur {:?}",
                //     stack,
                //     i + 1,
                //     current_best,
                //     new_actions,
                // );
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
        // add_cache(actions, &current_best, cached_recursives);
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
    let best = Best {
        earnings: zero!(),
        actions: Vec::new(),
        balance: balance.clone(),
    };

    fn recursive(
        best: Best,
        index: usize,
        // best: &mut Arc<Best>,
        data: &Vec<Row>,
    ) -> Best {
        if index >= data.len() {
            return best;
        }
        let row = data[index].clone();
        let skipped = recursive(best.clone(), index + 1, &data);
        if best.balance >= row.price {
            let mut new_actions = best.actions.clone();
            new_actions.push(index.clone());
            let new_best = Best {
                balance: best.balance - row.price,
                actions: new_actions.clone(),
                earnings: best.earnings + row.benefits,
            };
            let added = recursive(new_best, index + 1, &data);

            if added.earnings > skipped.earnings {
                return added;
            } else {
                return skipped;
            }
        } else {
            return skipped;
        }
    }
    Ok(recursive(best, 0, &data))
}

fn check_data(best: &Best, data: Vec<Row>, balance: Decimal) {
    let mut total_earnings = zero!();
    let mut balance = balance;
    println!("Best in test {:?}", best);
    for index_row in best.actions.clone() {
        let row = data[index_row].clone();
        total_earnings += row.benefits;
        balance -= row.price;
    }
    assert_eq!(best.balance, balance);
    assert!(balance >= zero!());
    assert_eq!(best.earnings, total_earnings);
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

fn curve_duration(algorithme: usize, data: Vec<Row>, balance: Decimal) -> Result<()> {
    let mut durations: Vec<i32> = Vec::new();
    let mut complexity: Vec<i32> = Vec::new();
    let mut nb_actions: Vec<usize> = Vec::new();
    let data_len = data.len();
    println!("datalen {}", data_len);
    let step;
    if data_len > 50 {
        step = 10;
    } else {
        step = 1;
    }
    for size in (2..data_len).step_by(step) {
        let safe_size = max(2, min(size, data_len));
        let n = safe_size as f64;
        let mut reduced_data = data[0..safe_size].to_vec();

        nb_actions.push(safe_size);
        let start = Instant::now();
        if algorithme != 1 {
            reduced_data.sort_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap());
            reduced_data.reverse();
        }
        run_algorithme(algorithme, reduced_data, balance)?;
        let end = Instant::now();
        let algo_duration = end.duration_since(start);
        println!("Plot {} : {:?}", safe_size, algo_duration);
        // durations.push(algo_duration.as_micros() as i32);
        durations.push(algo_duration.as_millis() as i32);
        // let p_complexity = 50.0 * n.log(10f64);
        // let p_complexity = 0.3 * n;
        let p_complexity = 2usize.pow(n as u32) as f64 * 0.0002;
        // let p_complexity = n * n.log(10f64);
        complexity.push((p_complexity) as i32);
    }

    let root_area =
        BitMapBackend::new("explanations/curve_brut_force.png", (600, 400)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Temps d'exécution / nombre d'actions", ("sans-serif", 30))
        .build_cartesian_2d(
            2..data_len + 1,
            0..((*durations.iter().max().unwrap() as f64 * 1.1) as i32),
            // 0..*durations.iter().max().unwrap(),
            // 0..durations[durations.len() - 1] * 2,
            // 0..(data_len + 100),
            // 0..((durations[durations.len() - 1] as f64 * 1.1) as i32),
        )?;

    ctx.configure_mesh()
        .x_desc("Nombre d'actions")
        .y_desc("Durée (ms)")
        .draw()
        .unwrap();

    ctx.draw_series(
        AreaSeries::new(
            (2..).step_by(step).zip(durations.iter().map(|x| *x)),
            0,
            &RED.mix(0.2),
        )
        .border_style(&RED),
    )?
    .label("brut_force")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    ctx.draw_series(
        AreaSeries::new(
            (2..).step_by(step).zip(complexity.iter().map(|x| *x)),
            0,
            &BLUE.mix(0.2),
        )
        .border_style(&BLUE),
    )?
    .label("O(k*2^n) with k = 1/5000")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    ctx.draw_series(PointSeries::of_element(
        (data_len - 4..data_len)
            .step_by(step)
            // durations starts x=2
            .map(|x| (x, durations[x - 2])),
        5,
        ShapeStyle::from(&RED).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + Text::new(
                    format!("{}: {:.2}s", coord.0, coord.1 as f64 / 1000.0),
                    (-69, -4),
                    ("sans-serif", 15),
                )
        },
    ))?;

    ctx.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    Ok(())
}

fn run_algorithme(algorithme: usize, data: Vec<Row>, balance: Decimal) -> Result<Best> {
    // Run the algorithme choosen in arguments
    Ok(match algorithme {
        0 => brut_force_recursive_binary(data.clone(), balance)?,
        1 => brut_force_recursive_redondant(data.clone(), balance)?,
        2 => optimized_recursive(data.clone(), balance)?,
        3 => optimized_one_loop(data.clone(), balance)?,
        4 => optimized_recursive_stack(data.clone(), balance)?,
        val => panic!("algorithme number {} does not exist.", val),
    })
}

fn main() -> Result<()> {
    // Parse arguments
    let args: Args = Args::parse();
    let balance: Decimal = args.balance.into();
    let dataset_number = args.dataset;
    let algorithme = args.algorithme;
    let curves = args.curves;

    let data: Vec<Row> = get_csv_dataset(dataset_number)?;
    // clean data : removes negative prices and profit
    let mut data: Vec<Row> = data
        .into_iter()
        .filter(|row| row.price > zero!() && row.profit > zero!())
        .collect();

    if curves {
        curve_duration(algorithme, data.clone(), balance)?;
    } else {
        // Start benchmark's clock
        let start = Instant::now();
        // sort data by profit (in pourcentage of the price)
        data.sort_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap());
        data.reverse();
        // Result benchmark's clock
        let end = Instant::now();
        let duration = end.duration_since(start);
        println!("Sorting duration : {:?}", duration);

        // Start benchmark's clock
        let start = Instant::now();

        let best = run_algorithme(algorithme, data.clone(), balance)?;

        // Result benchmark's clock
        let end = Instant::now();
        let duration = end.duration_since(start);
        show_result(data.clone(), &best, duration);

        // Verify the coherence of the result
        check_data(&best, data.clone(), balance);
    }

    Ok(())
}
