use super::RawCommand;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

/// Associate a command's name with a sender to the structure
/// where the command should be executed. If there is no sender
/// associated, then the command would be executed in the given
/// client fields.
pub struct CommandsMap {
    channel_map: HashMap<String, Vec<Option<Sender<Option<RawCommand>>>>>,
}

impl CommandsMap {
    /// Drops all the senders contained in the map.
    pub fn kill_senders(&mut self) {
        self.channel_map.iter_mut().for_each(|x| {
            let senders = x.1;
            senders.iter_mut().for_each(|x| {
                let _ = x.take();
            })
        })
    }

    /// Creates an empty instance of the commands map.
    pub fn new(
        channel_map: HashMap<String, Vec<Option<Sender<Option<RawCommand>>>>>,
    ) -> CommandsMap {
        CommandsMap { channel_map }
    }

    /// Returns the associated sender with the given command's name.
    pub fn get(&self, string: &str) -> Option<&Vec<Option<Sender<Option<RawCommand>>>>> {
        self.channel_map.get(string)
    }

    /// Creates an fully complete instance of the commands map.
    pub fn default(
        snd_cmd_dat: Sender<Option<RawCommand>>,
        snd_cmd_server: Sender<Option<RawCommand>>,
    ) -> CommandsMap {
        let mut channel_map: HashMap<String, Vec<Option<Sender<Option<RawCommand>>>>> =
            HashMap::new();

        Self::insert(
            &mut channel_map,
            vec![
                "clean".to_string(),
                "copy".to_string(),
                "del".to_string(),
                "exists".to_string(),
                "expire".to_string(),
                "expireat".to_string(),
                "keys".to_string(),
                "persist".to_string(),
                "rename".to_string(),
                "sort".to_string(),
                "touch".to_string(),
                "ttl".to_string(),
                "type".to_string(),
                "lindex".to_string(),
                "llen".to_string(),
                "lpop".to_string(),
                "lpush".to_string(),
                "lpushx".to_string(),
                "lrange".to_string(),
                "lrem".to_string(),
                "lset".to_string(),
                "rpop".to_string(),
                "rpush".to_string(),
                "rpushx".to_string(),
                "sadd".to_string(),
                "scard".to_string(),
                "sismember".to_string(),
                "smembers".to_string(),
                "srem".to_string(),
                "append".to_string(),
                "decrby".to_string(),
                "get".to_string(),
                "getdel".to_string(),
                "getset".to_string(),
                "incrby".to_string(),
                "mget".to_string(),
                "mset".to_string(),
                "set".to_string(),
                "strlen".to_string(),
                "dbsize".to_string(),
                "flushdb".to_string(),
                "save".to_string(),
            ],
            snd_cmd_dat.clone(),
        );

        // asociacion de comandos con server atributes

        Self::insert(
            &mut channel_map,
            vec![
                "pubsub".to_string(),
                "publish".to_string(),
                "config".to_string(),
                "notifymonitors".to_string(),
                "shutdown".to_string(),
            ],
            snd_cmd_server.clone(),
        );

        channel_map.insert(
            String::from("subscribe"),
            vec![None, Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("unsubscribe"),
            vec![None, Some(snd_cmd_server.clone())],
        );
        channel_map.insert(
            String::from("info"),
            vec![Some(snd_cmd_server), Some(snd_cmd_dat)],
        );
        channel_map.insert(String::from("monitor"), vec![None]);

        CommandsMap { channel_map }
    }

    fn insert(
        map: &mut HashMap<String, Vec<Option<Sender<Option<RawCommand>>>>>,
        commands: Vec<String>,
        sender: Sender<Option<RawCommand>>,
    ) {
        for cmd in commands.iter() {
            map.insert(String::from(cmd), vec![Some(sender.clone())]);
        }
    }
}
