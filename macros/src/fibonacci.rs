
//Fibonacci macro from the little book of rust macros 

macro_rules! multiply_add {
    ($a:expr, $b:expr, $c:expr) => {$a * ($b + $c)};
}

macro_rules! vec_strs {
   ( 
    //start a repitition
    $(
        //each repeat must contain an expression
        $element:expr
    )
    //separated by commas
    ,
    //number of times to be repeated - zero or more times
    *
) => {
    //Enclose the expansion in a block so that we can use 
    //multiple statements 
        {
            let mut v = Vec::new();
            
            //start a repitition
            $(
                //each repeat wil contain the following statement, with element replaced with the corresponding expression 
                v.push(format!("{}"), $element);
            )*

            v
        }
    };
}

macro_rules! capture_then_match_tokens {

}

//idea - gray code tower -> macro that takes in a list of numbers, generates their gray code and prints them in the command line vertically 

macro_rules! gray_code_tower {

}