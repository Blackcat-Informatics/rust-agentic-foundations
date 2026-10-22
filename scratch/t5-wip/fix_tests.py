import sys
p=sys.argv[1]
s=open(p).read()
old='''    let ws = lib("pub struct S { helper: u8 }\\n\\
         impl S { pub fn helper(&self) -> u8 { self.helper } }\\n\\
         pub enum E { V }\\n\\'''
new='''    let ws = lib("pub struct S { helper: u8 }\\n\\
         impl S { pub fn helper(&self) -> u8 { self.helper } }\\n\\
         pub mod kinds { pub enum E { V } }\\n\\'''
assert old in s, 1; s=s.replace(old,new,1)
old='''let label = 0; let _ = E::V; s.helper() + label }\\n");'''
new='''let label = 0; let _ = kinds::E::V; s.helper() + label }\\n");'''
assert old in s, 2; s=s.replace(old,new,1)
old='''    // `E::V` names the enum through its prefix, `S` the struct with its'''
new='''    // `kinds::E::V` names the enum through its prefix, `S` the struct with its'''
assert old in s, 3; s=s.replace(old,new,1)
old='''        [
            "demo::E",
            "demo::S",
            "demo::S::helper",
            "demo::api",
            "demo::label",
            "impl S:2",
        ]'''
new='''        [
            "demo::S",
            "demo::S::helper",
            "demo::api",
            "demo::kinds",
            "demo::kinds::E",
            "demo::label",
            "impl S:2",
        ]'''
assert old in s, 4; s=s.replace(old,new,1)
old='''    // Named alone, the module reaches nothing below it.'''
new='''    // A module that a path reaches first, below one that a root opens only
    // after the path has named both, is opened when contains reaches it.
    let ws = lib("pub mod r { pub mod a { pub mod m { pub mod inner { pub fn x() {} pub fn y() {} } } } }\\n\\
         pub fn f() { r::a::m::inner::x(); }\\n");
    let roots = [ty(&ws, "demo::r"), value(&ws, "demo::f")];
    let reached = reach(&ws, &roots);
    let deep = ty(&ws, "demo::r::a::m::inner");
    assert_eq!(
        reached.get(deep).map(|found| found.edge),
        Some(Edge::References)
    );
    assert!(reached.contains(value(&ws, "demo::r::a::m::inner::y")));
    let ws = lib("pub mod outer {\\n\\
             pub fn first() { super::outer::inner::x(); }\\n\\
             pub mod inner { pub fn x() {} pub fn y() {} }\\n\\
         }\\n");
    let inner = ty(&ws, "demo::outer::inner");
    // Named alone, the module reaches nothing below it.'''
assert old in s, 5; s=s.replace(old,new,1)
open(p,'w').write(s)
