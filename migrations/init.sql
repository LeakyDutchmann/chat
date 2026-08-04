create table if Not Exists messages(
room varchar(255) not null,
username varchar(255) not null,
message text not null
);

create table if Not Exists users(
id int unique primary key auto_increment,
username varchar(255) not null,
password_hash varchar(255) not null
);

create table if Not Exists session(
id int unique primary key auto_increment,
username varchar(255) not null,
session_id varchar(255) not null
);