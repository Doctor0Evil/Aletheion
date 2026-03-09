package aletheion.documentation.generator

const val DOC_GENERATOR_VERSION = 20260310L
const val MAX_DOCUMENTS = 65536
const val MAX_CODE_FILES = 262144
const val MAX_API_ENDPOINTS = 32768
const val TARGET_DOC_RATIO = 0.33

enum class DocumentType {
    API_REFERENCE, ARCHITECTURE, USER_GUIDE, ADMIN_GUIDE,
    DEPLOYMENT, TROUBLESHOOTING, COMPLIANCE, SECURITY,
    ACCESSIBILITY, INDIGENOUS_RIGHTS, BIOTIC_TREATY, ENVIRONMENTAL
}

enum class DocStatus {
    DRAFT, IN_REVIEW, APPROVED, PUBLISHED, DEPRECATED, ARCHIVED
}

data class CodeFile(
    val fileId: ULong,
    val filePath: String,
    val language: String,
    val lineCount: UInt,
    val codeLines: UInt,
    val commentLines: UInt,
    val blankLines: UInt,
    val docStrings: UInt,
    val functions: UInt,
    val classes: UInt,
    val complexity: UInt,
    val lastModifiedNs: Long,
    val documented: Boolean
) {
    fun docRatio(): Double = 
        if (lineCount > 0U) commentLines.toDouble() / lineCount.toDouble() else 0.0
    fun documentationCoverage(): Double = 
        if (functions + classes > 0U) docStrings.toDouble() / (functions + classes).toDouble() else 0.0
}

data class ApiEndpoint(
    val endpointId: ULong,
    val path: String,
    val method: String,
    val description: String,
    val parameters: List<String>,
    val responses: List<String>,
    val authenticated: Boolean,
    val rateLimited: Boolean,
    val documented: Boolean,
    val testCoverage: Double,
    val lastUpdatedNs: Long
)

data class Document(
    val documentId: ULong,
    val documentType: DocumentType,
    val title: String,
    val version: String,
    val status: DocStatus,
    val authorDid: String,
    val createdAtNs: Long,
    val updatedAtNs: Long,
    val publishedAtNs: Long?,
    val wordCount: UInt,
    val pagecount: UInt,
    val languages: List<String>,
    val relatedFiles: List<ULong>,
    val relatedEndpoints: List<ULong>,
    val reviewCount: UInt,
    val approvalCount: UInt,
    val accessibilityCompliant: Boolean,
    val indigenousReviewCompleted: Boolean,
    val bioticTreatyCompliant: Boolean
)

class DocumentationGenerator(
    private val generatorId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val codeFiles = mutableMapOf<ULong, CodeFile>()
    private val apiEndpoints = mutableMapOf<ULong, ApiEndpoint>()
    private val documents = mutableMapOf<ULong, Document>()
    private val auditLog = mutableListOf<DocAuditEntry>()
    private var nextFileId: ULong = 1UL
    private var nextEndpointId: ULong = 1UL
    private var nextDocumentId: ULong = 1UL
    private var totalCodeLines: UInt = 0U
    private var totalDocLines: UInt = 0U
    private var averageDocRatio: Double = 0.0
    private var documentationCompleteness: Double = 0.0
    private var lastGenerationNs: Long = initTimestampNs
    
    data class DocAuditEntry(
        val entryId: ULong,
        val action: String,
        val documentId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerCodeFile(file: CodeFile): Result<ULong> {
        if (codeFiles.size >= MAX_CODE_FILES) {
            logAudit("FILE_REGISTER", null, initTimestampNs, false, "File limit exceeded", 0.3)
            return Result.failure(Error("FILE_LIMIT_EXCEEDED"))
        }
        codeFiles[file.fileId] = file
        totalCodeLines += file.lineCount
        totalDocLines += file.commentLines
        logAudit("FILE_REGISTER", file.fileId, initTimestampNs, true, "File registered: ${file.filePath}", 0.02)
        return Result.success(file.fileId)
    }
    
    fun registerApiEndpoint(endpoint: ApiEndpoint): Result<ULong> {
        if (apiEndpoints.size >= MAX_API_ENDPOINTS) {
            return Result.failure(Error("ENDPOINT_LIMIT_EXCEEDED"))
        }
        apiEndpoints[nextEndpointId] = endpoint
        val endpointId = nextEndpointId
        nextEndpointId++
        logAudit("ENDPOINT_REGISTER", endpointId, initTimestampNs, true, "Endpoint registered: ${endpoint.path}", 0.02)
        return Result.success(endpointId)
    }
    
    fun createDocument(document: Document, nowNs: Long): Result<ULong> {
        if (documents.size >= MAX_DOCUMENTS) {
            return Result.failure(Error("DOCUMENT_LIMIT_EXCEEDED"))
        }
        if (!document.accessibilityCompliant) {
            logAudit("DOC_CREATE", null, nowNs, false, "Accessibility compliance required", 0.2)
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        documents[nextDocumentId] = document
        val docId = nextDocumentId
        nextDocumentId++
        logAudit("DOC_CREATE", docId, nowNs, true, "Document created: ${document.title}", 0.05)
        return Result.success(docId)
    }
    
    fun computeDocumentationRatio(): Double {
        if (totalCodeLines == 0U) return 0.0
        averageDocRatio = totalDocLines.toDouble() / totalCodeLines.toDouble()
        return averageDocRatio
    }
    
    fun computeDocumentationCompleteness(): Double {
        if (codeFiles.isEmpty()) return 0.0
        val documentedFiles = codeFiles.count { it.value.documented }
        val documentedEndpoints = apiEndpoints.count { it.value.documented }
        val fileCompleteness = documentedFiles.toDouble() / codeFiles.size
        val endpointCompleteness = documentedEndpoints.toDouble() / apiEndpoints.size.max(1).toDouble()
        documentationCompleteness = (fileCompleteness * 0.6 + endpointCompleteness * 0.4)
        return documentationCompleteness
    }
    
    fun generateApiDocumentation(endpointId: ULong, nowNs: Long): Result<ULong> {
        val endpoint = apiEndpoints[endpointId] ?: 
            return Result.failure(Error("ENDPOINT_NOT_FOUND"))
        if (!endpoint.documented) {
            return Result.failure(Error("ENDPOINT_NOT_DOCUMENTED"))
        }
        val doc = Document(
            documentId = nextDocumentId,
            documentType = DocumentType.API_REFERENCE,
            title = "API Reference: ${endpoint.path}",
            version = "1.0.0",
            status = DocStatus.DRAFT,
            authorDid = "SYSTEM",
            createdAtNs = nowNs,
            updatedAtNs = nowNs,
            publishedAtNs = null,
            wordCount = 1000U,
            pagecount = 5U,
            languages = listOf("en", "es", "oodham"),
            relatedFiles = emptyList(),
            relatedEndpoints = listOf(endpointId),
            reviewCount = 0U,
            approvalCount = 0U,
            accessibilityCompliant = true,
            indigenousReviewCompleted = false,
            bioticTreatyCompliant = true
        )
        return createDocument(doc, nowNs)
    }
    
    fun publishDocument(documentId: ULong, nowNs: Long): Result<Unit> {
        val document = documents[documentId] ?: 
            return Result.failure(Error("DOCUMENT_NOT_FOUND"))
        if (document.approvalCount < 2U) {
            return Result.failure(Error("INSUFFICIENT_APPROVALS"))
        }
        if (!document.indigenousReviewCompleted && 
            document.documentType in listOf(DocumentType.COMPLIANCE, DocumentType.BIOTIC_TREATY)) {
            return Result.failure(Error("INDIGENOUS_REVIEW_REQUIRED"))
        }
        val updatedDoc = document.copy(
            status = DocStatus.PUBLISHED,
            publishedAtNs = nowNs,
            updatedAtNs = nowNs
        )
        documents[documentId] = updatedDoc
        logAudit("DOC_PUBLISH", documentId, nowNs, true, "Document published", 0.05)
        return Result.success(Unit)
    }
    
    fun getGeneratorStatus(nowNs: Long): GeneratorStatus {
        val documentedFiles = codeFiles.count { it.value.documented }
        val documentedEndpoints = apiEndpoints.count { it.value.documented }
        val publishedDocs = documents.count { it.value.status == DocStatus.PUBLISHED }
        return GeneratorStatus(
            generatorId = generatorId,
            cityCode = cityCode,
            totalCodeFiles = codeFiles.size,
            documentedFiles = documentedFiles,
            totalApiEndpoints = apiEndpoints.size,
            documentedEndpoints = documentedEndpoints,
            totalDocuments = documents.size,
            publishedDocuments = publishedDocs,
            totalCodeLines = totalCodeLines,
            totalDocLines = totalDocLines,
            averageDocRatio = computeDocumentationRatio(),
            documentationCompleteness = computeDocumentationCompleteness(),
            lastGenerationNs = lastGenerationNs,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeDocumentationHealthScore(): Double {
        val docRatio = computeDocumentationRatio()
        val completeness = computeDocumentationCompleteness()
        val ratioScore = if (docRatio >= TARGET_DOC_RATIO) 1.0 else docRatio / TARGET_DOC_RATIO
        val publishedRatio = documents.count { it.value.status == DocStatus.PUBLISHED }.toDouble() / 
                            documents.size.coerceAtLeast(1).toDouble()
        return (ratioScore * 0.4 + completeness * 0.4 + publishedRatio * 0.2).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, documentId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = DocAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            documentId = documentId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<DocAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class GeneratorStatus(
    val generatorId: ULong,
    val cityCode: String,
    val totalCodeFiles: Int,
    val documentedFiles: Int,
    val totalApiEndpoints: Int,
    val documentedEndpoints: Int,
    val totalDocuments: Int,
    val publishedDocuments: Int,
    val totalCodeLines: UInt,
    val totalDocLines: UInt,
    val averageDocRatio: Double,
    val documentationCompleteness: Double,
    val lastGenerationNs: Long,
    val lastUpdateNs: Long
) {
    fun documentationReadiness(): Double {
        val fileRatio = documentedFiles.toDouble() / totalCodeFiles.coerceAtLeast(1).toDouble()
        val endpointRatio = documentedEndpoints.toDouble() / totalApiEndpoints.coerceAtLeast(1).toDouble()
        val publishRatio = publishedDocuments.toDouble() / totalDocuments.coerceAtLeast(1).toDouble()
        return (fileRatio * 0.4 + endpointRatio * 0.3 + publishRatio * 0.3).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixDocGenerator(generatorId: ULong, nowNs: Long): DocumentationGenerator {
    return DocumentationGenerator(generatorId, "PHOENIX_AZ", nowNs)
}
