#![feature(core_intrinsics)]

use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;

use crate::record::{load_relation, Relation};
use crate::util::ForceUnwrap;

mod common;
mod record;
mod util;

#[global_allocator]
static ALLOC: mimallocator::Mimalloc = mimallocator::Mimalloc;

// TODO: use aligned memory
fn calculate_query(relation: Relation, threshold: f32) -> u64 {
    let records = &relation.records;
    let record_lengths = relation.record_lengths.as_slice();

    records
        .iter()
        // .par_iter()
        .enumerate()
        .map(|(index, record)| {
            let index_first = index + 1;
            let following_record_count = relation.records.len() - index_first;

            // Gather intersection counts
            let mut intersections: Vec<u16> = vec![0; following_record_count];
            for value in record {
                let containing_records = &relation.index[value];
                let start_index = containing_records.partition_point(|&i| i <= index as u32);

                /*for &record_index in &containing_records[start_index..] {
                    unsafe {
                        let target_index = record_index as usize - index_first;
                        *intersections.get_unchecked_mut(target_index) += 1;
                    }
                }*/

                // Unrolled version
                let count = containing_records.len() - start_index;
                let unroll_count = count - (count % 8);

                for record_index in (start_index..start_index + unroll_count).step_by(8) {
                    unsafe {
                        let index0 =
                            *containing_records.get_unchecked(record_index) as usize - index_first;
                        let index1 = *containing_records.get_unchecked(record_index + 1) as usize
                            - index_first;
                        let index2 = *containing_records.get_unchecked(record_index + 2) as usize
                            - index_first;
                        let index3 = *containing_records.get_unchecked(record_index + 3) as usize
                            - index_first;
                        let index4 = *containing_records.get_unchecked(record_index + 4) as usize
                            - index_first;
                        let index5 = *containing_records.get_unchecked(record_index + 5) as usize
                            - index_first;
                        let index6 = *containing_records.get_unchecked(record_index + 6) as usize
                            - index_first;
                        let index7 = *containing_records.get_unchecked(record_index + 7) as usize
                            - index_first;
                        *intersections.get_unchecked_mut(index0) += 1;
                        *intersections.get_unchecked_mut(index1) += 1;
                        *intersections.get_unchecked_mut(index2) += 1;
                        *intersections.get_unchecked_mut(index3) += 1;
                        *intersections.get_unchecked_mut(index4) += 1;
                        *intersections.get_unchecked_mut(index5) += 1;
                        *intersections.get_unchecked_mut(index6) += 1;
                        *intersections.get_unchecked_mut(index7) += 1;
                    }
                }

                for &record_index in &containing_records[start_index + unroll_count..] {
                    unsafe {
                        let target_index = record_index as usize - index_first;
                        *intersections.get_unchecked_mut(target_index) += 1;
                    }
                }
            }

            // Vectorized JA
            /*let simd_count = following_record_count - (following_record_count % 8);
            let mut sum_v = f32x8::splat(0.0);
            let threshold_v = f32x8::splat(threshold);
            let zero_v = f32x8::splat(0.0);

            let intersections = intersections.as_slice();
            let relation_size_v = f32x8::splat(record.len() as f32);

            for index in (0..simd_count).step_by(8) {
                let intersection_v =
                    unsafe { u16x8::from_slice_unaligned_unchecked(intersections.slice(index, 8)) };
                let length_v = unsafe {
                    u16x8::from_slice_unaligned_unchecked(
                        record_lengths.slice(index_first + index, 8),
                    )
                };

                let intersection_v: u32x8 = intersection_v.into();
                let length_v: u32x8 = length_v.into();

                let intersection_v: f32x8 = intersection_v.cast();
                let length_v: f32x8 = length_v.cast();
                let union_v = length_v + relation_size_v - intersection_v;

                let ja_v = intersection_v / union_v;
                let mask = ja_v.ge(threshold_v);

                sum_v = sum_v + mask.select(intersection_v, zero_v);
            }*/
            let simd_count = 0;

            // Scalar JA for the rest of the elements
            intersections[simd_count..]
                .into_iter()
                .zip(&record_lengths[index_first + simd_count..])
                .map(|(&intersection, &length)| {
                    let union_size = (length as u32 + record.len() as u32) - intersection as u32;
                    let ja = (intersection as f32) / (union_size as f32);
                    if ja >= threshold {
                        intersection as u64
                    } else {
                        0
                    }
                })
                .sum::<u64>()
        })
        .sum()
}

fn main() {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(8)
    //     .build_global()
    //     .unwrap();

    // let stdin = std::fs::File::open("input").unwrap();
    // let stdin = BufReader::new(stdin);

    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let mut reader = BufReader::new(stdin);
    let mut line = String::with_capacity(64);

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    loop {
        // JS
        reader.read_line(&mut line).unwrap_force();
        if line.is_empty() {
            break;
        }

        line.clear();
        reader.read_line(&mut line).unwrap_force();

        let threshold = f32::from_str(&line.trim()).unwrap_force();

        line.clear();
        reader.read_line(&mut line).unwrap_force();

        let relation = crate::measure!("load", { load_relation(&line.trim()).unwrap() });
        let result = calculate_query(relation, threshold);
        writeln!(stdout, "{}", result).unwrap_force();

        line.clear();
    }
    stdout.flush().unwrap();
}
