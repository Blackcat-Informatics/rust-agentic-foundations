const fs=require('fs');
const names={
a_repository_that_is_not_judged_strict_is_exit_two_and_strict_adopts:'the_default_policy_can_record_a_current_structural_finding',
a_ceiling_row_and_its_class_decide_the_exit_of_strict_check_on_the_binary:'a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary',
};
for(const path of ['docs/capabilities/workplan-cli.md','docs/capabilities/workplan-strict.md']) {
 const text=fs.readFileSync(path,'utf8');
 const next=text.split(/(<!-- raf:generated:[\s\S]*?<!-- raf:generated:end -->)/).map((part,index)=>{
  if(index%2)return part;
  for(const [from,to]of Object.entries(names))part=part.replaceAll(from,to);
  return part;
 }).join('');
 fs.writeFileSync(path,next);
}
