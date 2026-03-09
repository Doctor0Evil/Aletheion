package aletheion.education.knowledge.graph

const val KNOWLEDGE_INDEXER_VERSION = 20260310L
const val MAX_NODES = 1000000L
const val MAX_EDGES = 5000000L
const val MAX_CONCEPTS = 100000L
const val INDEX_REFRESH_INTERVAL_S = 3600L

enum class NodeType {
    CONCEPT, DOCUMENT, PERSON, ORGANIZATION, LOCATION, EVENT, SKILL, COURSE,
    RESEARCH_PAPER, DATASET, SOFTWARE, HARDWARE, POLICY, REGULATION
}

enum class EdgeType {
    RELATES_TO, DERIVED_FROM, AUTHORED_BY, LOCATED_IN, PART_OF,
    PRECEDES, FOLLOWS, TEACHES, LEARNS, REQUIRES, ENABLES, RESTRICTS
}

data class KnowledgeNode(
    val nodeId: ULong,
    val nodeType: NodeType,
    val title: String,
    val description: String,
    val uri: String,
    val createdAtNs: Long,
    val updatedAtNs: Long,
    val version: UInt,
    val language: String,
    val accessLevel: UInt,
    val verified: Boolean,
    val sourceSystem: String,
    val metadata: Map<String, String>
) {
    fun isAccessible(userClearance: UInt): Boolean = accessLevel <= userClearance
    fun requiresVerification(): Boolean = !verified && nodeType in listOf(
        NodeType.RESEARCH_PAPER, NodeType.POLICY, NodeType.REGULATION
    )
}

data class KnowledgeEdge(
    val edgeId: ULong,
    val sourceNodeId: ULong,
    val targetNodeId: ULong,
    val edgeType: EdgeType,
    val weight: Double,
    val createdAtNs: Long,
    val confidence: Double,
    val verified: Boolean
) {
    fun isValid(): Boolean = confidence > 0.5 && weight > 0.0
    fun strength(): Double = weight * confidence
}

data class ConceptCluster(
    val clusterId: ULong,
    val primaryConcept: String,
    val relatedConcepts: List<String>,
    val nodeIds: List<ULong>,
    val centroid: Map<String, Double>,
    val createdAtNs: Long,
    val updatedAtNs: Long,
    val relevanceScore: Double
)

data class SearchQuery(
    val queryId: ULong,
    val queryString: String,
    val filters: Map<String, String>,
    val userClearance: UInt,
    val timestampNs: Long,
    val resultCount: Int,
    val executionTimeMs: Long
)

data class SearchResult(
    val nodeId: ULong,
    val title: String,
    val snippet: String,
    val relevanceScore: Double,
    val nodeType: NodeType,
    val uri: String,
    val accessGranted: Boolean
)

class KnowledgeGraphIndexer(
    private val indexerId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val nodes = mutableMapOf<ULong, KnowledgeNode>()
    private val edges = mutableMapOf<ULong, KnowledgeEdge>()
    private val conceptClusters = mutableMapOf<ULong, ConceptCluster>()
    private val searchHistory = mutableListOf<SearchQuery>()
    private var nextNodeId: ULong = 1UL
    private var nextEdgeId: ULong = 1UL
    private var nextClusterId: ULong = 1UL
    private var totalIndexingOperations: ULong = 0UL
    private var failedIndexingOperations: ULong = 0UL
    private var totalSearches: ULong = 0UL
    private var averageSearchTimeMs: Double = 0.0
    private var lastFullIndexNs: Long = initTimestampNs
    private var lastIncrementalIndexNs: Long = initTimestampNs
    
    fun addNode(node: KnowledgeNode): Result<ULong> {
        if (nodes.size >= MAX_NODES) {
            return Result.failure(Error("NODE_LIMIT_EXCEEDED"))
        }
        nodes[node.nodeId] = node
        nextNodeId = maxOf(nextNodeId, node.nodeId + 1UL)
        totalIndexingOperations++
        return Result.success(node.nodeId)
    }
    
    fun addEdge(edge: KnowledgeEdge): Result<ULong> {
        if (edges.size >= MAX_EDGES) {
            return Result.failure(Error("EDGE_LIMIT_EXCEEDED"))
        }
        if (!nodes.containsKey(edge.sourceNodeId) || !nodes.containsKey(edge.targetNodeId)) {
            failedIndexingOperations++
            return Result.failure(Error("INVALID_NODE_REFERENCE"))
        }
        edges[edge.edgeId] = edge
        nextEdgeId = maxOf(nextEdgeId, edge.edgeId + 1UL)
        totalIndexingOperations++
        return Result.success(edge.edgeId)
    }
    
    fun createConceptCluster(
        primaryConcept: String,
        relatedConcepts: List<String>,
        nodeIds: List<ULong>,
        nowNs: Long
    ): Result<ULong> {
        if (conceptClusters.size >= MAX_CONCEPTS) {
            return Result.failure(Error("CLUSTER_LIMIT_EXCEEDED"))
        }
        val cluster = ConceptCluster(
            clusterId = nextClusterId,
            primaryConcept = primaryConcept,
            relatedConcepts = relatedConcepts,
            nodeIds = nodeIds,
            centroid = computeCentroid(nodeIds),
            createdAtNs = nowNs,
            updatedAtNs = nowNs,
            relevanceScore = computeRelevanceScore(nodeIds)
        )
        conceptClusters[nextClusterId] = cluster
        nextClusterId++
        return Result.success(cluster.clusterId)
    }
    
    private fun computeCentroid(nodeIds: List<ULong>): Map<String, Double> {
        val centroid = mutableMapOf<String, Double>()
        for (nodeId in nodeIds) {
            val node = nodes[nodeId] ?: continue
            for ((key, value) in node.metadata) {
                val numericValue = value.toDoubleOrNull() ?: continue
                centroid[key] = (centroid[key] ?: 0.0) + numericValue
            }
        }
        for (key in centroid.keys) {
            centroid[key] = centroid[key]!! / nodeIds.size
        }
        return centroid
    }
    
    private fun computeRelevanceScore(nodeIds: List<ULong>): Double {
        if (nodeIds.isEmpty()) return 0.0
        var totalWeight = 0.0
        var totalConfidence = 0.0
        for (nodeId in nodeIds) {
            val nodeEdges = edges.values.filter { 
                it.sourceNodeId == nodeId || it.targetNodeId == nodeId 
            }
            for (edge in nodeEdges) {
                totalWeight += edge.weight
                totalConfidence += edge.confidence
            }
        }
        return (totalWeight + totalConfidence) / (nodeIds.size * 2)
    }
    
    fun search(
        queryString: String,
        filters: Map<String, String>,
        userClearance: UInt,
        nowNs: Long
    ): List<SearchResult> {
        val startTimeNs = System.nanoTime()
        val results = mutableListOf<SearchResult>()
        val queryTerms = queryString.lowercase().split(" ").filter { it.length > 2 }
        for ((nodeId, node) in nodes) {
            if (!node.isAccessible(userClearance)) continue
            var relevanceScore = 0.0
            for (term in queryTerms) {
                if (node.title.lowercase().contains(term)) relevanceScore += 0.5
                if (node.description.lowercase().contains(term)) relevanceScore += 0.3
                for ((key, value) in node.metadata) {
                    if (value.lowercase().contains(term)) relevanceScore += 0.2
                }
            }
            for ((filterKey, filterValue) in filters) {
                if (node.metadata[filterKey] != filterValue) {
                    relevanceScore = 0.0
                    break
                }
            }
            if (relevanceScore > 0.3) {
                results.add(SearchResult(
                    nodeId = nodeId,
                    title = node.title,
                    snippet = node.description.take(200),
                    relevanceScore = relevanceScore,
                    nodeType = node.nodeType,
                    uri = node.uri,
                    accessGranted = true
                ))
            }
        }
        results.sortByDescending { it.relevanceScore }
        val endTimeNs = System.nanoTime()
        val executionTimeMs = (endTimeNs - startTimeNs) / 1000000
        totalSearches++
        averageSearchTimeMs = (averageSearchTimeMs * (totalSearches - 1) + executionTimeMs) / totalSearches
        searchHistory.add(SearchQuery(
            queryId = totalSearches,
            queryString = queryString,
            filters = filters,
            userClearance = userClearance,
            timestampNs = nowNs,
            resultCount = results.size,
            executionTimeMs = executionTimeMs
        ))
        return results.take(50)
    }
    
    fun findRelatedNodes(nodeId: ULong, maxDepth: Int = 2): List<ULong> {
        val related = mutableSetOf<ULong>()
        val visited = mutableSetOf<ULong>()
        val queue = ArrayDeque<ULong>()
        queue.add(nodeId)
        visited.add(nodeId)
        var depth = 0
        while (queue.isNotEmpty() && depth < maxDepth) {
            val currentDepthSize = queue.size
            for (i in 0 until currentDepthSize) {
                val currentId = queue.removeFirst()
                for ((_, edge) in edges) {
                    val neighborId = if (edge.sourceNodeId == currentId) {
                        edge.targetNodeId
                    } else if (edge.targetNodeId == currentId) {
                        edge.sourceNodeId
                    } else {
                        null
                    }
                    if (neighborId != null && neighborId !in visited && edge.isValid()) {
                        related.add(neighborId)
                        visited.add(neighborId)
                        queue.add(neighborId)
                    }
                }
            }
            depth++
        }
        return related.toList()
    }
    
    fun computeNodeCentrality(nodeId: ULong): Double {
        val nodeEdges = edges.values.filter { 
            it.sourceNodeId == nodeId || it.targetNodeId == nodeId 
        }
        if (nodeEdges.isEmpty()) return 0.0
        return nodeEdges.sumOf { it.strength() } / nodeEdges.size
    }
    
    fun getIndexerStatus(nowNs: Long): IndexerStatus {
        val nodeTypeDistribution = nodes.values.groupingBy { it.nodeType }.eachCount()
        val edgeTypeDistribution = edges.values.groupingBy { it.edgeType }.eachCount()
        val avgNodeVersion = nodes.values.map { it.version }.average()
        val verificationRate = nodes.values.count { it.verified }.toDouble() / nodes.size
        return IndexerStatus(
            indexerId = indexerId,
            cityCode = cityCode,
            totalNodes = nodes.size.toLong(),
            totalEdges = edges.size.toLong(),
            totalClusters = conceptClusters.size.toLong(),
            nodeTypeDistribution = nodeTypeDistribution,
            edgeTypeDistribution = edgeTypeDistribution,
            averageNodeVersion = avgNodeVersion,
            verificationRate = verificationRate,
            totalIndexingOperations = totalIndexingOperations,
            failedIndexingOperations = failedIndexingOperations,
            totalSearches = totalSearches,
            averageSearchTimeMs = averageSearchTimeMs,
            lastFullIndexNs = lastFullIndexNs,
            lastIncrementalIndexNs = lastIncrementalIndexNs,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeKnowledgeGraphHealth(): Double {
        if (nodes.isEmpty()) return 0.0
        val connectivityScore = edges.size.toDouble() / nodes.size
        val verificationScore = nodes.values.count { it.verified }.toDouble() / nodes.size
        val freshnessScore = computeFreshnessScore(System.nanoTime())
        val searchEfficiencyScore = if (averageSearchTimeMs < 100) 1.0 else 0.7
        return (connectivityScore * 0.3 + verificationScore * 0.3 + 
                freshnessScore * 0.2 + searchEfficiencyScore * 0.2).coerceIn(0.0, 1.0)
    }
    
    private fun computeFreshnessScore(nowNs: Long): Double {
        val oneDayNs = 86400000000000L
        val recentNodes = nodes.values.count { nowNs - it.updatedAtNs < oneDayNs }
        return recentNodes.toDouble() / nodes.size
    }
    
    fun performIncrementalIndex(nowNs: Long) {
        lastIncrementalIndexNs = nowNs
        totalIndexingOperations++
    }
    
    fun performFullIndex(nowNs: Long) {
        lastFullIndexNs = nowNs
        lastIncrementalIndexNs = nowNs
        totalIndexingOperations++
    }
}

data class IndexerStatus(
    val indexerId: ULong,
    val cityCode: String,
    val totalNodes: Long,
    val totalEdges: Long,
    val totalClusters: Long,
    val nodeTypeDistribution: Map<NodeType, Int>,
    val edgeTypeDistribution: Map<EdgeType, Int>,
    val averageNodeVersion: Double,
    val verificationRate: Double,
    val totalIndexingOperations: ULong,
    val failedIndexingOperations: ULong,
    val totalSearches: ULong,
    val averageSearchTimeMs: Double,
    val lastFullIndexNs: Long,
    val lastIncrementalIndexNs: Long,
    val lastUpdateNs: Long
) {
    fun indexingSuccessRate(): Double {
        if (totalIndexingOperations == 0UL) return 1.0
        return (totalIndexingOperations - failedIndexingOperations).toDouble() / totalIndexingOperations
    }
    fun graphDensity(): Double {
        if (totalNodes <= 1) return 0.0
        return totalEdges.toDouble() / (totalNodes * (totalNodes - 1))
    }
}

fun createPhoenixKnowledgeIndexer(indexerId: ULong, nowNs: Long): KnowledgeGraphIndexer {
    return KnowledgeGraphIndexer(indexerId, "PHOENIX_AZ", nowNs)
}
