import `${DIR}/core` as core;

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul",  "Aug", "Sep", "Oct", "Nov", "Dec"];

fn parse_date(input) {
    let data = input.split('/');
    
    if data.len() == 3 {
        let day = parse_int(data[0]);
        let month = parse_int(data[1]);
        let year = parse_int(data[2]);
        
        if day in 1..31 && month in 1..12 && year in 1..2100 {
            return [day, month, year];
        }
    }
    
    return [];
}

// Main function
fn translate(input) {
    let data = core::parse_cmd(input);
    let date = parse_date(data);
    
    if !date.is_empty() {
        return [input, "", `${date[0]} ${global::MONTHS[date[1]]} ${date[2]}`, true];
    }
    
    return [input, "", "", false];
}

// Test
// print(translate(""));
// print(translate("01/02/2002"));
// print(translate("__"));
// print(translate("_04/06/2010_"));
// print(translate("_04/06/3000_"));
