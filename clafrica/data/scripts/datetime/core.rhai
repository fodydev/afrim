fn parse_cmd(input) {
    let data = input.split('_');
    
    if data.len() == 3 {
    	return data[1];
    }
    
    return "";
}
