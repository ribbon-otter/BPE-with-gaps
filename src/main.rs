use std::fs;
use std::collections::HashSet;
use counter::Counter;

//vocab[4] = the bytes of fifth token
//so you transalate u16 tokens into their bytes
//by indexing into the array
//and longer tokens are garenteed to have their prefixes
//occur before them
type Vocabulary = Vec<Vec<u8>>;

fn replace(doc : Vec<u16>, from : [u16; 2], to : u16) -> Vec<u16> {
	let mut result : Vec<u16> = Vec::with_capacity(doc.len());
	let mut i = 0;
	while i < doc.len() {
		if i + 1 < doc.len() &&
				[doc[i], doc[i+1]] == from {
			result.push(to);
			i += 1;
		} else {
			result.push(doc[i]);
		}
		i += 1;
	} 
	result
}

fn replace_with_gap(doc : Vec<u16>, from : [u16; 2], to : u16, gap : usize) -> Vec<u16> {
	let gop = gap + 1; //TODO think of better name for gap index ~summand
	let mut result : Vec<u16> = Vec::with_capacity(doc.len());
	let mut delay_line : u64 = 0; // shifted right each iteration, 
	                          // to delay binary signals
	                          // which control the deletion
	let mut i = 0;
	//println!("{delay_line:b} : {result:?} : {i}" );
	while i < doc.len() {
		if delay_line & 1 == 1 {
			//we already accounted for this token (so we do nothing)
		} else {
			if i + gop < doc.len()
					&& [doc[i], doc[i+gop]] == from {
				result.push(to);
				delay_line = delay_line | (1 << gop)
			} else {
				result.push(doc[i]);
			}
		}
		i += 1;
		delay_line >>= 1;
		//println!("{delay_line:b} : {result:?} : {i}" );
	}
	result
}


fn find_bpe_coding(data : &Vec<u8>, forbidden : &HashSet<u16>, vocab_size : u16) -> (Vocabulary, usize) {
	let mut doc: Vec<u16> = data.iter().map(|&b| b as u16).collect();
	let mut maps: Vocabulary = 
		Vec::with_capacity((vocab_size+256).into());
	
	//insert all the base tokens (they map to themselves)
	for i in 0u8..=255u8 {
		maps.push(vec!(i));
	}
	
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
		let expand_form : Vec<u8> = 
			most_common.iter().flat_map(
				|&t| maps[t as usize].iter()
			).copied().collect();
		//add this element to our vocabulary 
		maps.push(expand_form);
		doc = replace(doc, most_common, (maps.len()-1) as u16);
	}
	return (maps, doc.len());
}

fn count_gaps(doc : &Vec<u16>, forbidden : &HashSet<u16>, gop : usize) 
	-> Counter<[u16; 2]>
{
	//TODO no no no, the gop (i.e. gap+1 or a gap_size where you exclude one end) shows up here too
	let mut gap_counts: Counter<[u16; 2]> = Counter::new();
	for i in 0 .. doc.len()-gop {
		if doc[i..=i+gop].iter().any(|t| forbidden.contains(t)) {
			continue;
		}
		let pair : [u16; 2] = [doc[i], doc[i+gop]];
		gap_counts[&pair] += 1;
	}
	gap_counts
}

fn find_bpe_with_gaps_coding(data : &Vec<u8>,
		forbidden : &HashSet<u16>, 
		vocab_size : u16) -> (Vocabulary, usize) {
		const NEVER_VALID : u8 = 0xFF;
		const GAP : usize = 1;
		
		let mut doc: Vec<u16> = data.iter().map(|&b| b as u16).collect();
		let mut maps: Vocabulary = 
			Vec::with_capacity((vocab_size+256).into());
		
		//insert all the base tokens (they map to themselves)
		for i in 0u8..=255u8 {
			maps.push(vec!(i));
		}
	
	for _n in 0..vocab_size {
		let pair_counts: Counter<[u16; 2]> = count_gaps(&doc, &forbidden, 1);
		let gap_counts: Counter<[u16; 2]> = count_gaps(&doc, &forbidden, GAP+1);

		let most_common_pair : ([u16; 2], usize) =
				pair_counts.k_most_common_ordered(1).into_iter().next().unwrap();
		let most_common_gap : ([u16; 2], usize) =
				gap_counts.k_most_common_ordered(1).into_iter().next().unwrap();

		let expand_form : Vec<u8>;
		if most_common_gap.1 > most_common_pair.1 {
			expand_form = vec!(maps[most_common_gap.0[0] as usize].clone(),
			        vec!(NEVER_VALID), maps[most_common_gap.0[1] as usize].clone())
			        .concat();
			        //TODO the above line is wrong, because maybe the token we skipped over has an expanded form of more than one byte
			assert!(expand_form.len() >= 3);
			maps.push(expand_form);
			doc = replace_with_gap(doc, most_common_gap.0, (maps.len()-1) as u16, 1);
		} else {
			expand_form = most_common_pair.0.iter().flat_map(
				|&t| maps[t as usize].iter()
			).copied().collect();
			maps.push(expand_form);
			doc = replace(doc, most_common_pair.0, (maps.len()-1) as u16);
		}
		//add this element to our vocabulary 
	}
	return (maps, doc.len());
}

fn main() {
	let data: Vec<u8> = fs::read("./AliceInWonderland.txt").unwrap();
	let ( tokens2, length2 ) = find_bpe_coding(&data, &HashSet::from([' ' as u16]), 100);
	let ( tokens1, length1 ) = find_bpe_with_gaps_coding(&data, &HashSet::from([' ' as u16]), 100);
	println!("{length1} / {length2} = {}", (length1 as f64) / (length2 as f64));


	/* println!("constructed the following tokens:"); */
	/* for (token, code) in tokens.iter().enumerate() { */
	/* 	if code.len() == 1 { */
	/* 		continue; */
	/* 	} */
	/* 	let s =  String::from_utf8_lossy(code); */
	/* 	println!("{token}: {code:?} \"{s}\""); */
	/* } */
}

#[cfg(test)]
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
	fn test_replace_with_gap() {
		assert_eq!(replace_with_gap(vec!(1,2,1,0), [1,1], 3, 1), vec!(3,2,0));
	}
	#[test]
	fn test_replace_with_gap2() {
		assert_eq!(replace_with_gap(vec!(1,1,1,1), [1,1], 3, 1), vec!(3,3));
	}
	#[test]
	fn test_replace_with_gap3() {
		assert_eq!(replace_with_gap(vec!(0,0,1,1,1,1,0,1), [1,1], 3, 1), 
			vec!(0,0,3,3,0,1));
	}
	
	#[test]
	fn test_find_bpe_coding()  {
		let data : Vec<u8>  = vec!(1,2,1,2);
		let tokens = find_bpe_coding(&data, &HashSet::from([' ' as u16]), 1);
		assert_eq!(tokens.len(), 257);
		assert_eq!(tokens[256], vec!(1u8,2));
	}
	
	#[test]
	fn test_find_bpe_with_gaps_coding()  {
		let data : Vec<u8>  = vec!(1,2,1,3,1,4,1);
		let tokens = find_bpe_with_gaps_coding(&data, &HashSet::from([' ' as u16]), 1);
		assert_eq!(tokens.len(), 257);
		assert_eq!(tokens[256], vec!(1u8,0xFF,1));
	}
}

// vim: sw=2 ts=2
