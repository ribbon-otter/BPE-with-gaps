use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use counter::Counter;

fn replace(doc : Vec<u16>, from : [u16; 2], to : u16) -> Vec<u16> {
	let mut result : Vec<u16> = Vec::with_capacity(doc.len());
	let mut i = 0;
	while i < doc.len() {
		println!("{}",i);
		if i + 1 < doc.len() {
			let pair : [u16; 2] = [doc[i], doc[i+1]];
			if pair == from {
				result.push(to);
				i += 1;
			} else {
				result.push(doc[i]);
			}
		} else {
			result.push(doc[i]);
		}
		i += 1;
	} 
	result
}

fn find_bpe_coding(data : Vec<u8>, forbidden : HashSet<u16>, vocab_size : u16) -> HashMap<u16, Vec<u8>>{
	let mut doc: Vec<u16> = data.into_iter().map(|b| b as u16).collect();
	let mut maps: HashMap<u16, Vec<u8> > = HashMap::new();
	
	//insert all the base tokens (they map to themselves)
	for i in 0u8..255u8 {
		maps.insert(i.into(), vec!(i));
	}
	let mut next_vocab : u16 = 256;
	
	for _n in 0..vocab_size {
		let mut counts: Counter<[u16; 2]> = Counter::new();
		for i in 1..doc.len() {
			let pair : [u16; 2] = [doc[i-1], doc[i]];
			if forbidden.contains(&pair[0]) || forbidden.contains(&pair[1]) {
				continue; //ignore pairs which contain the forbidden tokens
			}
			counts[&pair] += 1;
		}
		let most_common : [u16; 2] =
					counts.k_most_common_ordered(1).into_iter().next().unwrap().0;
		//(but first find its ully expanded form)
		let expand_form : Vec<u8> = most_common.iter().flat_map(|t| maps.get(t).unwrap().iter() ).copied().collect();
		//add this element to our vocabulary 
		maps.insert(next_vocab, expand_form);
		doc = replace(doc, most_common, next_vocab);
		next_vocab += 1;
	}
	return maps;
}

fn main() {
	let data: Vec<u8> = fs::read("./AliceInWonderland.txt").unwrap();
	let tokens = find_bpe_coding(data, HashSet::from([' ' as u16]), 100);
	println!("constructed the following tokens:");
	for (token, code) in &tokens {
		if code.len() == 1 {
			continue;
		}
		let s =  String::from_utf8_lossy(code);
		println!("{token}: {code:?} \"{s}\"");
	}
}

mod tests {
	use super::*;
	
	#[test]
	fn test_replace() {
		assert_eq!(replace(vec!(1,2,1), [1,2], 3), vec!(3,1));
	}
	#[test]
	fn test_replace2() {
		assert_eq!(replace(vec!(1,2,1), [2,1], 3), vec!(1,3));
	}
	#[test]
	fn test_replace3() {
		assert_eq!(replace(vec!(1,2,1,2), [1,2], 3), vec!(3,3));
	}

	#[test]
	fn test_find_bpe_coding()  {
		let data : Vec<u8>  = vec!(1,2,1,2);
		let tokens = find_bpe_coding(data, HashSet::from([' ' as u16]), 1);
		assert_eq!(tokens.get(&256).unwrap(), &vec!(1u8,2));
	}
}

// vim: sw=2 ts=2
