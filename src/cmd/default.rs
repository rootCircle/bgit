use dialoguer::{theme::ColorfulTheme, MultiSelect};

pub(crate) fn default_cmd_workflow() {
    let multiselected = &[
        "Ice Cream",
        "Vanilla Cupcake",
        "Chocolate Muffin",
        "A Pile of sweet, sweet mustard",
    ];
    let defaults = &[false, false, true, false];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your food")
        .items(&multiselected[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("You selected these things:");
        for selection in selections {
            println!("  {}", multiselected[selection]);
        }
    }
}
