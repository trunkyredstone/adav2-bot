use serenity::model::channel::{GuildChannel, ChannelType};
use serenity::client::Context;
use serenity::model::guild::Member;

pub async fn move_vc(from: GuildChannel, to: GuildChannel, ctx: Context) {
    println!("Info  | Moving Users");
    if from.kind != ChannelType::Voice || to.kind != ChannelType::Voice {
        println!("Error | One of the specified channels is not a voice channel");
        return
    }

    let mut to_move: Vec<Member> = from.members(&ctx.cache).await.unwrap();

    while to_move.len() != 0 {
        for i in 0..to_move.len() {
            if let Err(why) = to_move[i].move_to_voice_channel(&ctx.http, to.id).await {
                println!("Error | Unable to move: {0}", why);
            }
            println!("Info  | Moved user");
        }

        to_move = from.members(&ctx.cache).await.unwrap();
    }
    println!("Info  | Done moving users");
}

pub async fn move_vc_filtered(from: GuildChannel, to: GuildChannel, filter: Vec<Member>, ctx: Context) {
    println!("Info  | Moving Users");
    if from.kind != ChannelType::Voice || to.kind != ChannelType::Voice {
        println!("Error | One of the specified channels is not a voice channel");
        return
    }

    let mut to_move: Vec<Member> = from.members(&ctx.cache).await.unwrap();

    while to_move.len() != (0 + filter.clone().len()) {
        for i in 0..to_move.len() {
            let mut can_move = false;

            for x in filter.clone() {
                if x.user.id == to_move[i].user.id {
                    can_move = true;
                }
            }

            if can_move {
                if let Err(why) = to_move[i].move_to_voice_channel(&ctx.http, to.id).await {
                    println!("Error | Unable to move: {0}", why);
                }
                println!("Info  | Moved user");
            }
        }

        to_move = from.members(&ctx.cache).await.unwrap();
    }
    println!("Info  | Done moving users");
}
