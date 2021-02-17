mod event_utils;

use config::Config;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use serenity::model::guild::{AuditLogEntry, AuditLogs, Member};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::user::User;
use mysql::prelude::*;
use mysql::{Pool};

struct Handler;

struct BotConfig;
struct DBConfig;
struct MemberList;

impl TypeMapKey for BotConfig {
    type Value = Config;
}
impl TypeMapKey for DBConfig {
    type Value = Pool;
}
impl TypeMapKey for MemberList {
    type Value = Vec<Member>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, _gid: GuildId, new_member: Member) {
        println!("Event | Join");

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        if let Err(why) = channel_id
            .say(&ctx, format!("✅ {} has joined | <@{}>", new_member.user.name, new_member.user.id))
            .await
        {
            println!("Error | {:?}", why)
        }
    }

    async fn guild_member_removal(&self, ctx: Context, _: GuildId, member: User, _: Option<Member>,) {
        println!("Event | Leave");

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Error | Config is wrong"));

        if let Err(why) = channel_id
            .say(&ctx, format!("❎ {} has left | <@{}>", member.name, member.id))
            .await
        {
            println!("Error | {:?}", why)
        }
    }

    async fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        println!("Event | Nickname change");

        let settings: Config = ctx
            .data
            .read()
            .await
            .get::<BotConfig>()
            .expect("Unable to find the config.")
            .clone();

        let channel_id = ChannelId(settings.get("welcome").expect("Error | Config is wrong"));

        let old_nick = old.to_owned().expect("No old nickname").nick;
        let new_nick = new.nick;
        let old_name = old.to_owned().expect("No old username").user.name;
        let new_name = new.user.name;

        if old_nick.is_none() && new_nick.is_some() {
            if let Err(why) = channel_id
                .say(&ctx, format!("ℹ️ {} ➡ ️{} | <@{}>", old_name, new_nick.expect("Couldn't get the new nickname"), new.user.id))
                .await
            {
                println!("Error | {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_some() {
            if old_nick.clone().expect("Couldn't get the old nickname") == new_nick.clone().expect("Couldn't get the new nickname") {
                return
            }

            if let Err(why) = channel_id
                .say(
                    &ctx,
                    format!("ℹ️ {} ➡ ️{} | <@{}>", old_nick.expect("Couldn't get the old nickname"), new_nick.expect("Couldn't get the new nickname"), new.user.id),
                )
                .await
            {
                println!("Error | {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_none() {
            if let Err(why) = channel_id
                .say(&ctx, format!("ℹ️ {} ➡ ️{} | <@{}>", old_nick.expect("Couldn't get the old nickname"), new_name, new.user.id))
                .await
            {
                println!("Error | {:?}", why)
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // let mut conn = ctx.data.read().await.get::<DBConfig>().expect("No DB cont").clone().get_conn().expect("Unable to get a connection");

        let content = msg.content;

        if content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error | {:?}", why)
            }
        }
        else if content.starts_with("!help") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Sorry, I can't help you.").await {
                println!("Error | {:?}", why)
            }
        }
        else if content.starts_with("!start"){
            let settings: Config = ctx
                .data
                .read()
                .await
                .get::<BotConfig>()
                .expect("Unable to find the config.")
                .clone();

            if msg.channel_id.as_u64() == &settings.get::<u64>("control-centre").unwrap() {
                println!("Event | Start event");
                let waiting_room = settings.get::<u64>("event-waiting").unwrap();
                let broadcasting_room = settings.get::<u64>("event-broadcasting").unwrap();
                let game_room = settings.get::<u64>("event-game").unwrap();

                let waiting_channel = ctx.cache.guild_channel(waiting_room).await.unwrap();
                let broadcasting_channel = ctx.cache.guild_channel(broadcasting_room).await.unwrap();
                let game_channel = ctx.cache.guild_channel(game_room).await.unwrap();

                if let Err(why) = msg.channel_id.start_typing(&ctx.http) {
                    println!("Error | {0}", why);
                }

                event_utils::move_vc(waiting_channel, game_channel.clone(), ctx.clone()).await;

                let broadcasting_members: Vec<Member> = broadcasting_channel.clone().members(&ctx.cache).await.unwrap();
                ctx.data.write().await.insert::<MemberList>(broadcasting_members);

                event_utils::move_vc(broadcasting_channel, game_channel, ctx.clone()).await;

                if let Err(why) = msg.channel_id.say(&ctx.http, "Moved users").await {
                    println!("Error | {0}", why);
                }
                // do other stuff
            }
        }
        else if content.starts_with("!end"){
            let settings: Config = ctx
                .data
                .read()
                .await
                .get::<BotConfig>()
                .expect("Unable to find the config.")
                .clone();

            if msg.channel_id.as_u64() == &settings.get::<u64>("control-centre").unwrap() {
                println!("Event | End event");
                let broadcasting_room = settings.get::<u64>("event-broadcasting").unwrap();
                let game_room = settings.get::<u64>("event-game").unwrap();
                let photo_room = settings.get::<u64>("event-photo").unwrap();

                let broadcasting_channel = ctx.cache.guild_channel(broadcasting_room).await.unwrap();
                let game_channel = ctx.cache.guild_channel(game_room).await.unwrap();
                let photo_channel = ctx.cache.guild_channel(photo_room).await.unwrap();

                if let Err(why) = msg.channel_id.start_typing(&ctx.http) {
                    println!("Error | {0}", why);
                }

                let member_list = ctx.data.read().await.get::<MemberList>().unwrap().clone();

                event_utils::move_vc_filtered(game_channel.clone(), broadcasting_channel, member_list, ctx.clone()).await;
                event_utils::move_vc(game_channel, photo_channel, ctx.clone()).await;

                if let Err(why) = msg.channel_id.say(&ctx.http, "Moved users").await {
                    println!("Error | {0}", why);
                }
                // do other stuff
            }
        }
        else if content.starts_with("!sendid"){

        }

        // let id = msg.author.id.as_u64().clone();
        //
        // // All in one statement:
        // // INSERT INTO levels VALUES (id, 0) ON DUPLICATE KEY UPDATE points = points + 1;
        // // TODO: Add a lookup for the points; Another values table?
        // // TODO: Blacklist for points?
        // // TODO: Role assigning and boundaries?
        //
        // let query = format!("INSERT INTO adadb.levels VALUES ({}, 1) ON DUPLICATE KEY UPDATE points = points + 1;", id);
        //
        // if let Err(_e) = conn.query_drop(query) {
        //     println!("Error | Unable to update points for user {}", id);
        // }
    }

    async fn message_delete(&self, ctx: Context, cid: ChannelId, msg: MessageId) {
        println!("Event | Message Deleted");

        let settings: Config = ctx
            .data
            .read()
            .await
            .get::<BotConfig>()
            .expect("Unable to find the config.")
            .clone();

        let to_ignore = settings
            .get_array("ignore_delete")
            .expect("Config is wrong");
        let to_ignore_iter = to_ignore.iter();
        for val in to_ignore_iter {
            if val.to_owned().into_str().expect("Config is wrong") == cid.as_u64().to_string() {
                println!("A message was deleted in a channel that should be ignored");
                return;
            }
        }

        let channel_id = ChannelId(settings.get("log").expect("Config is wrong"));
        let channel_id_self = ChannelId(settings.get("message-log").expect("Config is wrong"));

        let channel = cid.to_channel(&ctx).await.unwrap();
        let guild = channel.guild().unwrap();
        let gid = guild.guild_id;
        let audit_log: AuditLogs = gid
            .audit_logs(&ctx, Option::from(72_u8), None, None, Option::from(1_u8))
            .await
            .unwrap();
        let audit_entry: &AuditLogEntry = audit_log.entries.iter().next().unwrap().1;
        let user_id = audit_entry.user_id.as_u64();

        let message_maybe = ctx.cache.message(cid, msg).await;
        if message_maybe.clone().is_none() {
            if let Err(why) = channel_id.say(&ctx, format!("A message from someone was deleted in <#{}> by <@{}>, but it wasn't in the cache.",  cid.as_u64(), user_id)).await {
                println!("Error | {:?}", why)
            }
            return;
        }

        let message = message_maybe.clone().unwrap();
        let msg_content = message.content;

        if &audit_entry
            .target_id
            .expect("Unable to grab a target ID from the audit log")
            != message.author.id.as_u64()
        {
            if let Err(why) = channel_id_self
                .say(
                    &ctx,
                    format!(
                        "<@{}> deleted their message in <#{}>: {}",
                        message.author.id.as_u64(),
                        cid.as_u64(),
                        msg_content
                    ),
                )
                .await
            {
                println!("Client error {:?}", why)
            }
        } else {
            if let Err(why) = channel_id
                .say(
                    &ctx,
                    format!(
                        "A message from <@{}> was deleted in <#{}> by possibly <@{}>: {}",
                        message.author.id.as_u64(),
                        cid.as_u64(),
                        user_id,
                        msg_content
                    ),
                )
                .await
            {
                println!("Client error {:?}", why)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.cache.set_max_messages(1000).await;

        // Setup config
        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let mut data = ctx.data.write().await;
        data.insert::<BotConfig>(settings.clone());

        // Setup Database
        println!("Info  | Initialising database");
        let username: String = settings.get("db-username").unwrap();
        let password: String = settings.get("db-password").unwrap();
        let host: String = settings.get("db-host").unwrap();
        let db: String = settings.get("db-name").unwrap();
        let url = format!("mysql://{}:{}@{}/{}", username, password, host, db);
        let pool;
        let maybe_pool = mysql::Pool::new(&url);
        match maybe_pool {
            Err(_e) => {
                panic!("Error | Unable to connect to the database");
            }
            Ok(p) => {
                pool = p;
            }
        }
        let mut conn;
        let maybe_conn = pool.get_conn();
        match maybe_conn {
            Err(_e) => {
                panic!("Error | Unable to connect to the database");
            }
            Ok(c) => {
                conn = c;
            }
        }
        println!("Info  | Connected");
        // Test our connection is all good
        if let Err(_e) = conn.query_drop("UPDATE information SET times_started = times_started + 1;"){
            panic!("Error | Test query failed, unable to start");
        }

        data.insert::<DBConfig>(pool);

        println!("Info  | Context initialized");
        println!("Info  | {} is ready", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    println!("Info  | ADAv2: v1.1.8");
    println!("Info  | Initialising config");
    let mut settings = Config::default();
    settings.merge(config::File::with_name("Settings")).unwrap();

    let token: String = settings.get("token").unwrap();

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Error | {:?}", why);
    }
}
