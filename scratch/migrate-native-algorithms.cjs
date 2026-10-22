const fs=require('fs');
function body(s, name, replacement) {
 const from=s.indexOf(name); if(from<0)throw Error(name);
 const open=s.indexOf('{',from); let depth=1,end=open+1;
 while(depth && end<s.length){if(s[end]==='{')depth++;if(s[end]==='}')depth--;end++;}
 return s.slice(0,open)+'{\n'+replacement+'\n}'+s.slice(end);
}
let p='crates/raf-workplan/src/graph.rs',s=fs.readFileSync(p,'utf8');
s=body(s,'fn strongly_connected(', '    let mut groups = purrdf_core::graph::tarjan_scc(adjacent);\n    for group in &mut groups { group.sort_unstable(); }\n    groups.sort();\n    groups');
fs.writeFileSync(p,s);
p='crates/raf-source/src/settle.rs';s=fs.readFileSync(p,'utf8');
s=body(s,'fn components(', '    let index = purrdf_core::graph::scc_component_index(successors);\n    let count = index.iter().max().map_or(0, |maximum| maximum + 1);\n    let mut groups = vec![Vec::new(); count];\n    for (node, group) in index.into_iter().enumerate() { groups[group].push(node); }\n    groups');
s=s.replace("Tarjan's\n/// algorithm, with an explicit stack in place of recursion, so a long chain\n/// does not exhaust the caller's stack.", "The native component\n/// index supplies the reverse-topological ID law without caller recursion.");
fs.writeFileSync(p,s);
p='crates/raf-workplan/src/coverage/lines.rs';s=fs.readFileSync(p,'utf8');
s=body(s,'fn times(', '    use purrdf_xsd::bigint::BigInt;\n    BigInt::from_decimal_digits(digits).expect("validated decimal digits")\n        .mul(&BigInt::from_u128(factor)).to_string()');
fs.writeFileSync(p,s);
p='crates/raf-workplan/src/digest.rs';s=fs.readFileSync(p,'utf8');
const start=s.indexOf('const INITIAL:');const end=s.indexOf('#[cfg(test)]',start);
s=s.slice(0,start)+s.slice(end);
s=s.replace('use std::str::FromStr;', 'use std::str::FromStr;\nuse sha2::{Digest as _, Sha256};');
s=s.replace('SHA-256 is the function of FIPS 180-4, implemented here with no\n//! dependency.', "SHA-256 uses the same streaming implementation as PurRDF; its public\n//! content-store API supplies one-shot hashing, not a streaming state.");
s=s.replaceAll('hasher.finish()', 'hasher.finalize().into()');
s=s.replace('for byte in self.0 {\n            write!(f, "{byte:02x}")?;\n        }\n        Ok(())','fmt::Display::fmt(&purrdf_hash::hex::Lower(&self.0), f)');
const a=s.indexOf('        let digits = text.as_bytes();',s.indexOf('impl FromStr for Digest'));
const b=s.indexOf('        Ok(Self(bytes))',a)+'        Ok(Self(bytes))'.length;
s=s.slice(0,a)+'        purrdf_hash::hex::decode_32_canonical(text).map(Self).ok_or_else(refuse)'+s.slice(b);
s=s.replace('    use std::fmt::Write as _;', '    use sha2::Digest as _;');
s=body(s,'    fn hex(bytes:', '        purrdf_hash::hex::encode(bytes)');
s=s.replace('hex(&hasher.finalize().into())', 'hex(hasher.finalize().as_slice())');
fs.writeFileSync(p,s);
p='crates/raf-workplan/src/strict/claims.rs';s=fs.readFileSync(p,'utf8');
s=s.replace('use crate::digest::Sha256;\n','');
s=s.replace('let mut hash = Sha256::new();\n    hash.update(folded.as_bytes());', 'let hash = purrdf_core::content_store::ContentDigest::of(folded.as_bytes());');
s=s.replace('for byte in &hash.finish()[..DIGEST_DIGITS / 2] {\n        write!(key, "{byte:02x}").expect("a string takes text");\n    }','purrdf_hash::hex::encode_into(&hash.as_bytes()[..DIGEST_DIGITS / 2], &mut key);');
fs.writeFileSync(p,s);

