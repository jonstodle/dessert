alias ue := unpack-episode
alias um := unpack-movie
alias cd := clean-destination

unpack-episode:
  cargo run -- -s ./test-data/episode -d ./test-data/destination
  ls -halF ./test-data/destination

unpack-movie:
  cargo run -- -s ./test-data/movie -d ./test-data/destination
  ls -halF ./test-data/destination

clean-destination:
  rm -rf ./test-data/destination/*
