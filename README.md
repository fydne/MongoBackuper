<p>
<a href="#">
<img src="https://raw.githubusercontent.com/fydne/MongoBackuper/main/bins/build.ico" width="128px" align="right" style="border-radius: 50%;" />
</a>

# Mongo Backuper
<p align="center">
<a href="#">
<img src="https://readme-typing-svg.herokuapp.com?font=Fira+Code&weight=500&pause=1000&color=2EF733&center=true&vCenter=true&repeat=false&random=false&width=435&height=25&lines=Mongo+Backuper">
</a>
</p>
<p align="center">
Create backups of MongoDB
</p>

# Info
Creates a backup of the Mongo database in the `/MongoBackups` directory (on Windows - `C:/MongoBackups`)

To restore the backup use the [mongorestore](https://github.com/mongodb/mongo-tools/tree/master/mongorestore) utility


# Configs
The config file is located in `MongoBackups/config.js`

> Although the file has a .js extension, use the .json syntax

Modify the `config.js` file.

```js
[
    {
        "name": "mydb", // The name of the database (for the backup directory), can be arbitrary
        "url": "mongodb://localhost", // Link-connect to MongoDB
        "interval": 4, // Sets the interval for database backup (in hours)
        "removeOld": 30 // Automatically deletes old backups that exceed the specified number of backups (In days) *But keeps one backup in any occasions.
    },
    { // To backup multiple databases
        "name": "mydb2",
        "url": "mongodb://user:password@host:port",
        "interval": 12,
        "removeOld": 15
    }
]
```

<p align="center">
<a href="#">
<img src="https://profile-counter.glitch.me/mongo_backuper/count.svg" width="200px" />
</a>