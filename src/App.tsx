import { useState } from 'react'
import {
  AppBar,
  Toolbar,
  Typography,
  Container,
  Paper,
  Button,
  Box,
  Grid,
  Card,
  CardContent,
  CardActions,
  Alert,
  LinearProgress,
  Chip,
  List,
  ListItem,
  ListItemText
} from '@mui/material'
import {
  FolderOpen as FolderIcon,
  Analytics as AnalyticsIcon,
  Download as DownloadIcon,
  Architecture as ArchitectureIcon
} from '@mui/icons-material'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'

interface AnalysisResult {
  graph: {
    metrics: {
      total_capsules: number
      total_relations: number
      complexity_average: number
      coupling_index: number
      cohesion_index: number
      cyclomatic_complexity: number
      depth_levels: number
    }
    capsules: Record<string, any>
    relations: any[]
    layers: Record<string, string[]>
  }
  warnings: any[]
  recommendations: string[]
}

function App() {
  const [projectPath, setProjectPath] = useState<string>('')
  const [isAnalyzing, setIsAnalyzing] = useState(false)
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null)
  const [error, setError] = useState<string>('')

  const selectProject = async () => {
    try {
      const selected = await open({
        directory: true,
        title: 'Выберите папку проекта для анализа'
      })
      
      if (selected && typeof selected === 'string') {
        setProjectPath(selected)
        setError('')
      }
    } catch (err) {
      setError('Ошибка при выборе папки: ' + String(err))
    }
  }

  const analyzeProject = async () => {
    if (!projectPath) {
      setError('Сначала выберите папку проекта')
      return
    }

    setIsAnalyzing(true)
    setError('')
    
    try {
      console.log('Начинаем анализ проекта:', projectPath)
      
      // Валидируем путь
      await invoke('validate_project_path', { projectPath })
      
      // Запускаем анализ
      const result = await invoke('analyze_project', { 
        projectPath,
        config: null 
      }) as string
      
      console.log('Результат анализа:', result)
      
      // Парсим JSON результат
      const parsedResult = JSON.parse(result) as AnalysisResult
      setAnalysisResult(parsedResult)
      
    } catch (err) {
      console.error('Ошибка анализа:', err)
      setError('Ошибка анализа: ' + String(err))
    } finally {
      setIsAnalyzing(false)
    }
  }

  const exportAnalysis = async (format: string) => {
    try {
      const exported = await invoke('export_analysis', { format }) as string
      
      // Создаем и скачиваем файл
      const blob = new Blob([exported], { type: 'text/plain' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `analysis.${format}`
      a.click()
      URL.revokeObjectURL(url)
      
    } catch (err) {
      setError('Ошибка экспорта: ' + String(err))
    }
  }

  const generateDiagram = async () => {
    try {
      const diagram = await invoke('generate_architecture_diagram') as string
      
      // Создаем и скачиваем файл
      const blob = new Blob([diagram], { type: 'text/plain' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'architecture.md'
      a.click()
      URL.revokeObjectURL(url)
      
    } catch (err) {
      setError('Ошибка генерации диаграммы: ' + String(err))
    }
  }

  return (
    <Box sx={{ flexGrow: 1, minHeight: '100vh', bgcolor: 'background.default' }}>
      <AppBar position="static" elevation={0}>
        <Toolbar>
          <ArchitectureIcon sx={{ mr: 2 }} />
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            ArchLens - Анализатор архитектуры кода
          </Typography>
        </Toolbar>
      </AppBar>

      <Container maxWidth="lg" sx={{ mt: 4, pb: 4 }}>
        {/* Выбор проекта */}
        <Paper sx={{ p: 3, mb: 3 }}>
          <Typography variant="h5" gutterBottom>
            Выбор проекта
          </Typography>
          
          <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
            <Button
              variant="outlined"
              startIcon={<FolderIcon />}
              onClick={selectProject}
              disabled={isAnalyzing}
            >
              Выбрать папку
            </Button>
            
            {projectPath && (
              <Typography variant="body2" color="text.secondary">
                {projectPath}
              </Typography>
            )}
          </Box>

          <Button
            variant="contained"
            startIcon={<AnalyticsIcon />}
            onClick={analyzeProject}
            disabled={!projectPath || isAnalyzing}
            size="large"
          >
            {isAnalyzing ? 'Анализируем...' : 'Анализировать проект'}
          </Button>

          {isAnalyzing && (
            <Box sx={{ mt: 2 }}>
              <LinearProgress />
              <Typography variant="body2" sx={{ mt: 1 }}>
                Сканирование файлов и построение графа архитектуры...
              </Typography>
            </Box>
          )}
        </Paper>

        {/* Ошибки */}
        {error && (
          <Alert severity="error" sx={{ mb: 3 }}>
            {error}
          </Alert>
        )}

        {/* Результаты анализа */}
        {analysisResult && (
          <Grid container spacing={3}>
            {/* Метрики */}
            <Grid item xs={12} md={6}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Метрики архитектуры
                  </Typography>
                  
                  <List dense>
                    <ListItem>
                      <ListItemText
                        primary="Общее количество модулей"
                        secondary={analysisResult.graph.metrics.total_capsules}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="Количество связей"
                        secondary={analysisResult.graph.metrics.total_relations}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="Средняя сложность"
                        secondary={analysisResult.graph.metrics.complexity_average.toFixed(2)}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="Индекс связанности"
                        secondary={analysisResult.graph.metrics.coupling_index.toFixed(2)}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="Индекс сплоченности"
                        secondary={analysisResult.graph.metrics.cohesion_index.toFixed(2)}
                      />
                    </ListItem>
                  </List>
                </CardContent>
              </Card>
            </Grid>

            {/* Слои архитектуры */}
            <Grid item xs={12} md={6}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Архитектурные слои
                  </Typography>
                  
                  {Object.entries(analysisResult.graph.layers).map(([layer, modules]) => (
                    <Box key={layer} sx={{ mb: 1 }}>
                      <Chip 
                        label={`${layer}: ${modules.length} модулей`}
                        variant="outlined"
                        size="small"
                        sx={{ mr: 1 }}
                      />
                    </Box>
                  ))}
                </CardContent>
              </Card>
            </Grid>

            {/* Рекомендации */}
            {analysisResult.recommendations.length > 0 && (
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>
                      Рекомендации по улучшению
                    </Typography>
                    
                    {analysisResult.recommendations.map((rec, index) => (
                      <Alert key={index} severity="info" sx={{ mb: 1 }}>
                        {rec}
                      </Alert>
                    ))}
                  </CardContent>
                </Card>
              </Grid>
            )}

            {/* Экспорт */}
            <Grid item xs={12}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Экспорт результатов
                  </Typography>
                </CardContent>
                <CardActions>
                  <Button 
                    startIcon={<DownloadIcon />}
                    onClick={() => exportAnalysis('json')}
                  >
                    JSON
                  </Button>
                  <Button 
                    startIcon={<DownloadIcon />}
                    onClick={() => exportAnalysis('yaml')}
                  >
                    YAML
                  </Button>
                  <Button 
                    startIcon={<DownloadIcon />}
                    onClick={generateDiagram}
                  >
                    Диаграмма Mermaid
                  </Button>
                </CardActions>
              </Card>
            </Grid>
          </Grid>
        )}
      </Container>
    </Box>
  )
}

export default App 