pub use ext_core::Project;
use ext_error::{AppError, Result};
use sea_orm::{Database as SeaOrmDatabase, DbConn};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct Database {
    db: DbConn,
    projects_dir: PathBuf,
}

impl Database {
    pub async fn new(db_url: &str, projects_dir: &str) -> Result<Self> {
        // Initialize database
        let db = SeaOrmDatabase::connect(db_url)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Ensure projects directory exists
        let projects_path = PathBuf::from(projects_dir);
        fs::create_dir_all(&projects_path)
            .await
            .map_err(|e| AppError::Database(format!("Failed to create projects directory: {}", e)))?;

        Ok(Self {
            db,
            projects_dir: projects_path,
        })
    }

    pub async fn save_project(&self, project: &Project) -> Result<()> {
        // Save project metadata to database
        self.save_project_to_db(project).await?;

        // Save project folder structure
        self.save_project_to_filesystem(project).await?;

        Ok(())
    }

    async fn save_project_to_db(&self, project: &Project) -> Result<()> {
        // TODO: Implement Sea-ORM entity operations here
        // This will be done after setting up migrations and entities
        Ok(())
    }

    async fn save_project_to_filesystem(&self, project: &Project) -> Result<()> {
        let project_path = self.projects_dir.join(project.id.to_string());
        
        // Create project directory
        fs::create_dir_all(&project_path)
            .await
            .map_err(|e| AppError::Database(format!("Failed to create project directory: {}", e)))?;

        // Save project metadata as JSON
        let metadata = serde_json::json!({
            "id": project.id.to_string(),
            "name": project.name,
            "description": project.description,
            "created_at": project.created_at.to_rfc3339(),
            "updated_at": project.updated_at.to_rfc3339(),
        });

        let metadata_path = project_path.join("project.json");
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata)
                .map_err(|e| AppError::Database(format!("Failed to serialize project: {}", e)))?,
        )
        .await
        .map_err(|e| AppError::Database(format!("Failed to write project file: {}", e)))?;

        Ok(())
    }

    pub async fn load_project(&self, project_id: &str) -> Result<Option<Project>> {
        let project_path = self.projects_dir.join(project_id).join("project.json");
        
        if !project_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&project_path)
            .await
            .map_err(|e| AppError::Database(format!("Failed to read project file: {}", e)))?;

        let project: Project = serde_json::from_str(&content)
            .map_err(|e| AppError::Database(format!("Failed to parse project: {}", e)))?;

        Ok(Some(project))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        let mut entries = fs::read_dir(&self.projects_dir)
            .await
            .map_err(|e| AppError::Database(format!("Failed to read projects directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AppError::Database(format!("Failed to read directory entry: {}", e)))?
        {
            if let Ok(Some(project)) = self.load_project(
                entry
                    .file_name()
                    .to_string_lossy()
                    .as_ref(),
            )
            .await
            {
                projects.push(project);
            }
        }

        Ok(projects)
    }
}