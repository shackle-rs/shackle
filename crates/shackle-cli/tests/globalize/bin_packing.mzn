% Bin packing: the load of each bin, written as sums over the items
int: n_items;
int: n_bins;
array[1..n_items] of int: weight;
array[1..n_items] of var 1..n_bins: bin;
array[1..n_bins] of var 0..sum(weight): load;
constraint forall(b in 1..n_bins)(load[b] = sum(i in 1..n_items)(weight[i] * (bin[i] = b)));
