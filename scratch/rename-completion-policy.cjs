const fs = require('node:fs');
const cp = require('node:child_process');
const source = fs.readFileSync('crates/raf-workplan/tests/complete.rs', 'utf8');
const names = [...source.matchAll(/^fn (under_strict_[a-z0-9_]+)\(/gm)].map(m => m[1]);
const pairs = names.map(old => [old, old.replace(/^under_strict_/, '')
    .replace(/_and_under_default_it_completes$/, '')
    .replace(/_refuse_as_under_default_and_their_repairs_complete$/, '_refuse_until_their_repairs_complete')]);
pairs.push(['under_default_a_none_change_that_edits_a_bound_item_owes_nothing_and_under_strict_it_owes_one', 'a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired']);
pairs.push(['refused_under_strict_and_completed_under_default', 'refuses_public_token_changes_until_the_effect_is_declared']);
const paths = cp.execFileSync('rg', ['--files', 'crates', 'docs', '--glob', '*.rs', '--glob', '*.md'], {encoding: 'utf8'}).trim().split('\n');
for (const path of paths) {
    const before = fs.readFileSync(path, 'utf8');
    let after = before;
    for (const [old, replacement] of pairs) after = after.split(old).join(replacement);
    if (after !== before) fs.writeFileSync(path, after);
}
process.stdout.write(pairs.map(([old, replacement]) => `${old} -> ${replacement}`).join('\n') + '\n');
