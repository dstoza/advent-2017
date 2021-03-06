#![deny(clippy::all, clippy::pedantic)]
#![feature(test)]

extern crate test;

use std::convert::TryInto;

use bit_set::BitSet;
use clap::{crate_name, App, Arg};
use common::LineReader;

enum Direction {
    East,
    Southeast,
    Southwest,
    West,
    Northwest,
    Northeast,
}

impl Direction {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Direction::East,
            1 => Direction::Southeast,
            2 => Direction::Southwest,
            3 => Direction::West,
            4 => Direction::Northwest,
            5 => Direction::Northeast,
            _ => panic!("Unexpected direction index {}", index),
        }
    }
}

struct DirectionIterator<'a> {
    line: &'a str,
    cursor: usize,
}

impl<'a> DirectionIterator<'a> {
    fn new(line: &'a str) -> Self {
        Self { line, cursor: 0 }
    }
}

impl<'a> Iterator for DirectionIterator<'a> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == self.line.len() {
            return None;
        }

        let first = &self.line[self.cursor..=self.cursor];
        match first {
            "e" => {
                self.cursor += 1;
                Some(Direction::East)
            }
            "s" => {
                let next = &self.line[self.cursor + 1..=self.cursor + 1];
                self.cursor += 2;
                match next {
                    "e" => Some(Direction::Southeast),
                    "w" => Some(Direction::Southwest),
                    _ => panic!("Unexpected character after 's': {}", next),
                }
            }
            "w" => {
                self.cursor += 1;
                Some(Direction::West)
            }
            "n" => {
                let next = &self.line[self.cursor + 1..=self.cursor + 1];
                self.cursor += 2;
                match next {
                    "w" => Some(Direction::Northwest),
                    "e" => Some(Direction::Northeast),
                    _ => panic!("Unexpected character after 'n': {}", next),
                }
            }
            _ => panic!("Unexpected first character: {}", first),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i8,
    y: i8,
}

impl Coordinate {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    fn from_address(address: usize) -> Self {
        let x: i16 = ((address >> 8) & 0xFF).try_into().unwrap();
        let y: i16 = (address & 0xFF).try_into().unwrap();

        Self {
            x: (x - 128).try_into().unwrap(),
            y: (y - 128).try_into().unwrap(),
        }
    }

    fn step(&mut self, direction: &Direction) {
        match direction {
            Direction::East => self.x += 2,
            Direction::Southeast => {
                self.y -= 2;
                self.x += 1;
            }
            Direction::Southwest => {
                self.y -= 2;
                self.x -= 1;
            }
            Direction::West => self.x -= 2,
            Direction::Northwest => {
                self.y += 2;
                self.x -= 1;
            }
            Direction::Northeast => {
                self.y += 2;
                self.x += 1;
            }
        }
    }

    fn get_address(self) -> u16 {
        let high_byte: u16 = (i16::from(self.x) + 128).try_into().unwrap();
        let low_byte: u16 = (i16::from(self.y) + 128).try_into().unwrap();
        high_byte << 8 | low_byte
    }
}

fn get_coordinate(line: &str) -> Coordinate {
    let mut coordinate = Coordinate::new();
    for direction in DirectionIterator::new(line) {
        coordinate.step(&direction);
    }
    coordinate
}

fn get_adjacent_tiles(coordinate: Coordinate) -> [Coordinate; 6] {
    let mut adjacent_tiles = [coordinate; 6];
    for (index, direction) in (0..6).map(Direction::from_index).enumerate() {
        adjacent_tiles[index].step(&direction);
    }
    adjacent_tiles
}

fn count_adjacent_black_tiles(coordinate: Coordinate, black_tiles: &BitSet) -> usize {
    let adjacent_tiles = get_adjacent_tiles(coordinate);
    let mut count = 0;
    for adjacent_tile in &adjacent_tiles {
        if black_tiles.contains(adjacent_tile.get_address() as usize) {
            count += 1;
            if count > 2 {
                return count;
            }
        }
    }
    count
}

fn evolve_tiles(black_tiles: &mut BitSet) {
    let mut tiles_to_flip = Vec::new();
    let mut white_tiles = BitSet::new();

    for black_tile in black_tiles.iter() {
        let coordinate = Coordinate::from_address(black_tile);
        let adjacent_black_tile_count = count_adjacent_black_tiles(coordinate, black_tiles);
        if adjacent_black_tile_count == 0 || adjacent_black_tile_count > 2 {
            tiles_to_flip.push(black_tile);
        }

        for adjacent_tile in &get_adjacent_tiles(coordinate) {
            white_tiles.insert(adjacent_tile.get_address() as usize);
        }
    }

    white_tiles.difference_with(black_tiles);
    for white_tile in &white_tiles {
        let coordinate = Coordinate::from_address(white_tile);
        let adjacent_black_tile_count = count_adjacent_black_tiles(coordinate, black_tiles);
        if adjacent_black_tile_count == 2 {
            tiles_to_flip.push(white_tile);
        }
    }

    for tile_to_flip in tiles_to_flip {
        if !black_tiles.remove(tile_to_flip) {
            black_tiles.insert(tile_to_flip);
        }
    }
}

fn main() {
    let args = App::new(crate_name!())
        .arg(Arg::from_usage("<FILE>"))
        .get_matches();

    let mut black_tiles = BitSet::new();

    let mut reader = LineReader::new(args.value_of("FILE").unwrap());
    reader.read_with(|line| {
        let coordinate = get_coordinate(line);
        if !black_tiles.remove(coordinate.get_address() as usize) {
            black_tiles.insert(coordinate.get_address() as usize);
        }
    });

    println!("{} tiles remain flipped", black_tiles.len());

    let days = 100;
    for _day in 1..=days {
        evolve_tiles(&mut black_tiles);
    }

    println!("After {} days, {} tiles are black", days, black_tiles.len());
}

#[cfg(test)]
mod tests {
    // use test::Bencher;
}
