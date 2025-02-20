cargo zigbuild --release --target x86_64-unknown-linux-gnu

export IP=18.197.17.99
scp -i ~/.ssh/mvp-key-pair-server.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/network-http ec2-user@$IP:/home/ec2-user/
scp -i ~/.ssh/mvp-key-pair-server.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/disk-read ec2-user@$IP:/home/ec2-user/
scp -i ~/.ssh/mvp-key-pair-server.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/network ec2-user@$IP:/home/ec2-user/
scp -i ~/.ssh/mvp-key-pair-server.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/network_disk ec2-user@$IP:/home/ec2-user/

scp -i ~/.ssh/mvp-key-pair-client.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/client ec2-user@$IP:/home/ec2-user/
scp -i ~/.ssh/mvp-key-pair-client.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/microbenchmarks/target/x86_64-unknown-linux-gnu/release/client-http ec2-user@$IP:/home/ec2-user/
export IP=3.126.153.240
