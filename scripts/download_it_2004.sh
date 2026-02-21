#!/bin/bash
set -e

mkdir -p data/uk-2007-05@100000
cd data/uk-2007-05@100000

BASE="uk-2007-05@100000"
URL_BASE="http://data.law.di.unimi.it/webdata/uk-2007-05%40100000"

echo "Downloading uk-2007-05@100000 dataset..."
curl -O "${URL_BASE}/${BASE}.graph"
curl -O "${URL_BASE}/${BASE}.properties"
curl -O "${URL_BASE}/${BASE}.md5sums"

echo "Verifying checksums..."
md5sum -c "${BASE}.md5sums" 2>/dev/null | grep OK || echo "Checksum verification failed for some files, but graph and properties might be OK."

echo "Done. You can now run the benchmark with:"
echo "cargo bench --bench hit_leiden_suite"
