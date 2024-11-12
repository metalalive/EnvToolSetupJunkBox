## Deep dive into internal architecture
- [Elasticsearch from the Bottom Up, Part 1](https://www.elastic.co/blog/found-elasticsearch-from-the-bottom-up)
- [keeping elasticsearch in sync with rational database](https://www.elastic.co/blog/found-keeping-elasticsearch-in-sync)
- [Lucene internal - java subreddit](https://www.reddit.com/r/java/comments/11igbyn/comment/jay7l5b)

Highlight of the articles :
* When designing a replication strategy, the two most important concerns to consider are : **acceptable replication delay** and **data consistency**
* Perfect synchronization between an application’s primary datastore (e.g. rational database) and Elasticsearch is rarely needed, and seldom possible.
* Elasticsearch index is composed of multiple Lucene indexes, Each Lucene index is in turn composed of multiple **segments** inside of which documents reside.
* Lucene segments are essentially immutable collections of documents
* When an update is made to a document:
  * the old document is marked as deleted in its existing segment
  * the new document is buffered and used to create a new segment
* which degrades performance , all analyzers must be re-run for documents whose values change, incurring potentially high CPU utilization
* when the number of segments in the index has grown excessively, and/or the ratio of deleted documents in a segment is high, multiple segments are merged into a new single segment by copying documents out of old segments and into a new one, after which the old segments are deleted.
* the most versatile way to bulk updates to documents in Elasticsearch is to use a queue with some sort of uniqueness constraint. The basic idea is to define an acceptable interval between document updates and to update the document no more frequently than that interval.
* 
