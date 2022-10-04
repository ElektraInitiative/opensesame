extern crate elektra;

use elektra::{KeySet, LookupOption, StringKey, WriteableKey, ReadableKey, KDB, CopyOption};
use std::str::FromStr;
use std::collections::HashMap;

pub struct Config<'a> {
	kdb: KDB,
	parent_key: StringKey<'a>,
	ks: KeySet,
}

impl Config<'_> {
	pub fn new(parent: &str) -> Self {
		let mut s = Self {
			kdb: KDB::open(KeySet::with_capacity(0)).unwrap(),
			parent_key: StringKey::new(parent).unwrap(),
			ks: KeySet::with_capacity(100),
		};
		s.sync();
		s
	}

	pub fn sync(&mut self) {
		let res = self.kdb.get(&mut self.ks, &mut self.parent_key);
		match res {
			Ok(_success) => (),
			Err(kdb_error) => {
				panic!("Set config failed: {}", kdb_error.to_error_message());
			}
		}
	}

	#[cfg(test)]
	pub fn cut(&mut self, name: &str) {
		let cut_key = StringKey::new(&format!("user:/{}/{}", self.parent_key.name(), name)).unwrap();
		self.ks.cut(&cut_key);
	}

	pub fn add(&mut self, name: &str, value: &str) {
		let mut new_key = StringKey::new(&format!("user:/{}/{}", self.parent_key.name(), name)).unwrap();
		new_key.set_value(value);
		self.ks.append_key(new_key);
	}

	pub fn set(&mut self, name: &str, value: &str) {
		self.add(name, value);
		let res = self.kdb.set(&mut self.ks, &mut self.parent_key);
		match res {
			Ok(_success) => (),
			Err(kdb_error) => {
				panic!("Set config failed: {}", kdb_error.to_error_message());
			}
		}
	}

	pub fn get_hash_map_vec_u8(&mut self, name: &str) -> HashMap<Vec<u8>,String> {
		let mut lookup_key = self.parent_key.duplicate(CopyOption::KEY_CP_NAME);
		lookup_key.add_name(name).expect(format!("Could not add '{}' to '{}'!", name, self.parent_key.name()).as_str());
		let mut ret = HashMap::new();
		for (_i, key) in self.ks.iter_mut().enumerate() {
			if key.is_directly_below(&lookup_key) {
				ret.insert(key.value()
						.replace(&['(', ')', ',', '[', ']'][..], "")
						.split_whitespace()
						.map(|s| s.parse().unwrap())
						.collect(),
					  key.basename().to_string());
			}
		}
		return ret;
	}

	pub fn get_bool(&mut self, name: &str) -> bool {
		let mut lookup_key = self.parent_key.duplicate(CopyOption::KEY_CP_NAME);
		lookup_key.add_name(name).expect(format!("Could not add '{}' to '{}'!", name, self.parent_key.name()).as_str());
		if let Some(found_key) =
			self.ks.lookup(lookup_key, LookupOption::KDB_O_NONE)
		{
			return found_key.value().to_string() == "1";
		}
		return false;
	}

	pub fn get_option<T: FromStr>(&mut self, name: &str) -> Option<T> {
		let mut lookup_key = self.parent_key.duplicate(CopyOption::KEY_CP_NAME);
		lookup_key.add_name(name).expect(format!("Could not add '{}' to '{}'!", name, self.parent_key.name()).as_str());
		if let Some(found_key) =
			self.ks.lookup(lookup_key, LookupOption::KDB_O_NONE)
		{
			if let Ok(ret) =
				found_key.value().parse::<T>()
			{
				return Some(ret);
			}
		}
		None
	}

	pub fn get<T: FromStr>(&mut self, name: &str) -> T {
		let mut lookup_key = self.parent_key.duplicate(CopyOption::KEY_CP_NAME);
		lookup_key.add_name(name).expect(format!("Could not add '{}' to '{}'!", name, self.parent_key.name()).as_str());
		if let Some(found_key) =
			self.ks.lookup(lookup_key, LookupOption::KDB_O_NONE)
		{
			if let Ok(ret) =
				found_key.value().parse::<T>()
			{
				return ret;
			} else {
				panic!("Could not convert '{}' to type '{}' from key '{}'!", found_key.value(), std::any::type_name::<T>(), found_key.name());
			}
		} else {
			panic!("Did not find the key '{}/{}'!", self.parent_key.name(), name);
		}
	}

}
