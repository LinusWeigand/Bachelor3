cargo zigbuild --release --target x86_64-unknown-linux-gnu

export IP=3.121.206.198
scp -i ~/.ssh/mvp-key-pair-server.pem /Users/linusweigand/Documents/CodeProjects/rust/Bachelor/Bachelor3/v1/target/x86_64-unknown-linux-gnu/release/v1 ec2-user@$IP:/home/ec2-user/
