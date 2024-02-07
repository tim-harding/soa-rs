#!/usr/bin/jq -rf

select(has("target")) 
| select(
    .target?.kind? 
    | map(. == "bench") 
    | any
) 
| .executable
