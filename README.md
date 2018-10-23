# alco
Basic alleleCounter implementation

Build:

    cargo build --release

Run:

    ./alco -b data.bam -l loci.txt --minbasequal 20 --minmapqual 35 --required-flag 2 --filtered-flag 1796
